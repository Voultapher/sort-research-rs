use std::cmp;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::benchmark::{evalute_sort, BenchmarkResult, BenchmarkResultKey};
use crate::measure::DurationOpaque;

pub trait Sort {
    fn name() -> String;
    fn sort<T: Ord>(v: &mut [T]);
}

pub enum CompareResult {
    NoBaseline,
    NoSignificantChange,
    Regression,
    Improvement,
}

/// If `base_line_path` points to a file that exists, compares to base-line. Creates a base-line
/// and stores it to `base_line_path` otherwise.
pub fn compare_sort<S: Sort>(base_line_path: &Path) -> CompareResult {
    let existing_baseline = fs::read_to_string(base_line_path)
        .map(|existing_baseline_str| {
            serde_json::from_str::<BenchmarkResult>(&existing_baseline_str).unwrap()
        })
        .ok();

    let new_results = evalute_sort::<S>();
    let new_results_serialized = serde_json::to_string(&new_results).unwrap();

    let new_base_line_path = if existing_baseline.is_some() {
        change_file_name(
            base_line_path,
            &format!(
                "{}_new",
                base_line_path.file_stem().unwrap().to_str().unwrap()
            ),
        )
    } else {
        base_line_path.to_owned()
    };

    fs::write(&new_base_line_path, new_results_serialized).unwrap();
    println!(
        "Wrote new baseline to file: {}\n",
        new_base_line_path.display()
    );

    if let Some(old_baseline) = existing_baseline {
        if old_baseline.version != new_results.version {
            panic!(
                "Version mimatch, baseline: {} new: {}",
                old_baseline.version, new_results.version
            );
        }

        compare_results(&old_baseline, &new_results)
    } else {
        CompareResult::NoBaseline
    }
}

fn compare_results(baseline: &BenchmarkResult, new: &BenchmarkResult) -> CompareResult {
    // Across all data points, weighted equally the total difference must exceed this to be viewed
    // as significant. During testing a median of 0.5% with a max of 1% was observed when testing
    // the same binary and sort implementation on a relatively noise free machine. This is
    // deliberately higher than the max observed to avoid false positives.
    const MIN_SIGNIFICANT_TOTAL_SPEEDUP: f64 = 0.02;

    // Filter out some noise when reporting what changed.
    const MIN_SIGNIFICANT_INDIVIDUAL_SPEEDUP: f64 = 0.05;

    // Assuming some baseline noise, this noise should be distributed across patterns somewhat
    // evenly. But it's possible that a single pattern shows a significant perf change, that would
    // be smothered by the relatively high MIN_SIGNIFICANT_TOTAL_SPEEDUP bar.
    const MIN_SIGNIFICANT_PATTERN_SPEEDUP: f64 = 0.07;

    let mut commulative_speedup = 0.0;
    let mut significant_changes = HashMap::new();
    let mut pattern_specific_speedups = HashMap::new();

    let baseline_name = baseline
        .results
        .iter()
        .map(|(key, _)| key.sort_name())
        .next()
        .unwrap();
    let new_name = new
        .results
        .iter()
        .map(|(key, _)| key.sort_name())
        .next()
        .unwrap();

    for (bench_key, old_duration) in &baseline.results {
        let compare_name = BenchmarkResultKey::new(format!(
            "{}-{}-{}-{}-{}",
            new_name,
            bench_key.predicition_state(),
            bench_key.ty(),
            bench_key.pattern(),
            bench_key.len()
        ));

        let new_duration = new.results.get(&compare_name).expect(&format!(
            "Result key not found in new results: {}",
            bench_key.full_name()
        ));

        let relative_speedup = relative_speedup(*new_duration, *old_duration);

        commulative_speedup += relative_speedup;

        pattern_specific_speedups
            .entry(format!(
                "{}-{}-{}",
                bench_key.predicition_state(),
                bench_key.ty(),
                bench_key.pattern()
            ))
            .or_insert(Vec::new())
            .push((bench_key.len(), relative_speedup));

        if relative_speedup.abs() >= MIN_SIGNIFICANT_INDIVIDUAL_SPEEDUP {
            significant_changes.insert(bench_key, relative_speedup);
        }
    }

    let total_speedup = commulative_speedup / baseline.results.len() as f64;

    println!("Comparing {new_name} to baseline {baseline_name}:");

    // let mut individual_change_info = String::from("\nIndividual changes: [");
    let mut significant_changes_sorted = significant_changes.into_iter().collect::<Vec<_>>();
    significant_changes_sorted
        .sort_by(|(_, a_speedup), (_, b_speedup)| a_speedup.abs().total_cmp(&b_speedup.abs()));

    let individual_changes = significant_changes_sorted
        .into_iter()
        .rev()
        .map(|(bench_key, speedup)| {
            let times_x_change = relative_speedup_as_times_x(speedup);
            format!(
                "{}-{}-{}-{}: {:.3}x",
                bench_key.predicition_state(),
                bench_key.ty(),
                bench_key.pattern(),
                bench_key.len(),
                times_x_change
            )
        })
        .collect::<Vec<_>>();

    println!(
        "\nIndividual changes: [{}]\n",
        individual_changes.join(", ".into())
    );

    // This is necessary because we don't know the HashMap iter order.
    for (_, relative_speedups) in &mut pattern_specific_speedups {
        relative_speedups.sort_unstable_by_key(|(len, _)| *len);
    }

    // Track them separately to avoid a situation where the total speedup is an improvement, but
    // below the MIN_SIGNIFICANT_TOTAL_SPEEDUP threshold, and some other heuristic says one aspect
    // is a regression.
    let mut perf_improvements = 0;
    let mut perf_regressions = 0;

    let mut handle_sub_check_result = |compare_result: CompareResult| match compare_result {
        CompareResult::Improvement => {
            perf_improvements += 1;
        }
        CompareResult::Regression => {
            perf_regressions += 1;
        }
        _ => (),
    };

    handle_sub_check_result(check_full_regression(
        "Total",
        total_speedup,
        MIN_SIGNIFICANT_TOTAL_SPEEDUP,
    ));

    for (name, relative_speedups) in pattern_specific_speedups {
        let pattern_specific_total_speedup = relative_speedups
            .iter()
            .map(|(_, speedup)| speedup)
            .sum::<f64>()
            / relative_speedups.len() as f64;

        // It's possible that multiple checks contradict earlier results, but that's valuable
        // information and we should surface that information to the user.
        handle_sub_check_result(check_full_regression(
            &name,
            pattern_specific_total_speedup,
            MIN_SIGNIFICANT_PATTERN_SPEEDUP,
        ));
        handle_sub_check_result(check_consistent_regression(&name, &relative_speedups));
        handle_sub_check_result(check_window_regression(&name, &relative_speedups));
    }

    println!(
        "Debug total_speedup: {:.3}x",
        relative_speedup_as_times_x(total_speedup)
    );

    let (compare_result, change_str) = if perf_regressions != 0 || perf_improvements != 0 {
        if perf_regressions > perf_improvements
            || (perf_regressions == perf_improvements && total_speedup.is_sign_negative())
        {
            (CompareResult::Regression, "Regression")
        } else {
            (CompareResult::Improvement, "Improvement")
        }
    } else {
        (CompareResult::NoSignificantChange, "No significant change")
    };

    println!("\nResult: {change_str}");

    compare_result
}

#[must_use]
fn check_full_regression(name: &str, speedup: f64, threshold: f64) -> CompareResult {
    if speedup.abs() >= threshold {
        // Abs because the direction is indicated by the language, and might be confusing.
        let times_x_change = relative_speedup_as_times_x(speedup).abs();

        if speedup.is_sign_negative() {
            println!("{name} REGRESSED by {times_x_change:.3}x");
            return CompareResult::Regression;
        }

        println!("{name} IMPROVED by {times_x_change:.3}x");
        return CompareResult::Improvement;
    }

    CompareResult::NoSignificantChange
}

#[must_use]
fn check_consistent_regression(name: &str, speedups: &[(usize, f64)]) -> CompareResult {
    // This can help find changes that only affect certain patterns, that by themselves won't
    // clear the other significance thresholds.

    // If `REQUIRED_PATTERN_CONSISTENCY_PERCENT` of a pattern are all above this threshold in the
    // same direction, it is seen as an improvement or regression.
    const MIN_SIGNIFICANT_CONSISTENT_PATTERN_SPEEDUP: f64 = 0.03;
    const REQUIRED_PATTERN_CONSISTENCY_PERCENT: f64 = 70.0;

    let (consistent_improvement_count, consistent_regression_count) =
        count_consistent_changes(&speedups, MIN_SIGNIFICANT_CONSISTENT_PATTERN_SPEEDUP);

    let min_change_len =
        (speedups.len() as f64 * (REQUIRED_PATTERN_CONSISTENCY_PERCENT / 100.0)).round() as usize;
    let change_percent = ((cmp::max(consistent_regression_count, consistent_improvement_count)
        as f64
        / speedups.len() as f64)
        * 100.0)
        .round();

    if consistent_improvement_count >= min_change_len {
        println!("{name} IMPROVED because {change_percent}% of the data points improved.");
        return CompareResult::Improvement;
    }

    if consistent_regression_count >= min_change_len {
        println!("{name} REGRESSED because {change_percent}% of the data points regressed.");
        return CompareResult::Regression;
    }

    CompareResult::NoSignificantChange
}

#[must_use]
fn check_window_regression(name: &str, speedups: &[(usize, f64)]) -> CompareResult {
    // This can help find changes that only affect certain areas, that by themselves won't clear
    // the other significance thresholds.

    // If `PATTERN_OUTLIER_WINDOW` points for a pattern all exceed this threshold it is seen as an
    // improvement or regression.
    const MIN_SIGNIFICANT_PATTERN_OUTLIER_SPEEDUP: f64 = 0.10;
    const PATTERN_OUTLIER_WINDOW: usize = 5;

    for window in speedups.windows(PATTERN_OUTLIER_WINDOW) {
        let (consistent_improvement_count, consistent_regression_count) =
            count_consistent_changes(window, MIN_SIGNIFICANT_PATTERN_OUTLIER_SPEEDUP);

        assert!(
            consistent_improvement_count + consistent_regression_count <= PATTERN_OUTLIER_WINDOW
        );

        if consistent_improvement_count == PATTERN_OUTLIER_WINDOW {
            println!(
                "{name} IMPROVED because {PATTERN_OUTLIER_WINDOW} contiguous values improved, lens: {:?}.", window.iter().map(|(len, _)| len).collect::<Vec<_>>()
            );
            return CompareResult::Improvement;
        }

        if consistent_regression_count == PATTERN_OUTLIER_WINDOW {
            println!(
                "{name} REGRESSED because {PATTERN_OUTLIER_WINDOW} contiguous values regressed, lens: {:?}.",window.iter().map(|(len, _)| len).collect::<Vec<_>>()
            );
            return CompareResult::Regression;
        }
    }

    CompareResult::NoSignificantChange
}

fn count_consistent_changes(speedups: &[(usize, f64)], threshold: f64) -> (usize, usize) {
    let consistent_improvement_count = speedups
        .iter()
        .map(|(_, speedup)| (*speedup >= threshold) as usize)
        .sum::<usize>();

    let consistent_regression_count = speedups
        .iter()
        .map(|(_, speedup)| (*speedup <= -threshold) as usize)
        .sum::<usize>();

    (consistent_improvement_count, consistent_regression_count)
}

/// If time_a is faster than time_b -> % faster than time_b
/// If time_b is faster than time_a -> % faster than time_a as negative number
/// 100 == time_a 2x faster than time_b
/// -100 == time_b 2x faster than time_a
fn relative_speedup(time_a: DurationOpaque, time_b: DurationOpaque) -> f64 {
    if time_a <= time_b {
        // time_a is faster.
        (time_b.as_opaque() / time_a.as_opaque()) - 1.0
    } else {
        // time_b is faster
        -((time_a.as_opaque() / time_b.as_opaque()) - 1.0)
    }
}

fn relative_speedup_as_times_x(relative_speedup: f64) -> f64 {
    relative_speedup + if relative_speedup >= 0.0 { 1.0 } else { -1.0 }
}

fn change_file_name(path: impl AsRef<Path>, name: &str) -> PathBuf {
    let path = path.as_ref();
    let mut result = path.to_owned();
    result.set_file_name(name);
    if let Some(ext) = path.extension() {
        result.set_extension(ext);
    }
    result
}
