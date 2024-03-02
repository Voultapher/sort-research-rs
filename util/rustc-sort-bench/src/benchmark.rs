use std::cmp;
use std::collections::HashMap;
use std::hint::black_box;

use serde::{Deserialize, Serialize};

use crate::measure::{measure_duration, DurationOpaque};
use crate::patterns;
use crate::Sort;

/// By versioning the baseline files, we can catch compatibility issues early.
const BENCHMARK_RESULT_VERSION: usize = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub version: usize,
    pub results: HashMap<BenchmarkResultKey, DurationOpaque>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BenchmarkResultKey {
    full_name: String,
}

impl BenchmarkResultKey {
    pub fn new(full_name: String) -> Self {
        assert_eq!(full_name.split('-').count(), 5);

        Self { full_name }
    }

    pub fn full_name(&self) -> &str {
        &self.full_name
    }

    pub fn sort_name(&self) -> &str {
        self.part(0)
    }
    pub fn predicition_state(&self) -> &str {
        self.part(1)
    }
    pub fn ty(&self) -> &str {
        self.part(2)
    }
    pub fn pattern(&self) -> &str {
        self.part(3)
    }
    pub fn len(&self) -> usize {
        self.part(4).parse().unwrap()
    }

    fn part(&self, idx: usize) -> &str {
        self.full_name.split('-').nth(idx).unwrap()
    }
}

impl Serialize for DurationOpaque {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_opaque().serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for DurationOpaque {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        Ok(Self::from_opaque(f64::deserialize(deserializer)?))
    }
}

pub fn evalute_sort<S: Sort>() -> BenchmarkResult {
    let mut results = HashMap::new();

    // Pin the benchmark to the same core to improve repeatability. Doing it this way allows
    // criterion to do other stuff with other threads, which greatly impacts overall benchmark
    // throughput.
    pin_thread_to_core();

    run_type_benchmarks::<S, u64>("u64", u64::from, &mut results);
    run_type_benchmarks::<S, F128>("f128", F128::new, &mut results);

    // String is highly allocator and layout dependent, and is not reliable enough. So it is
    // disabled by default.
    #[cfg(feature = "string_bench")]
    run_type_benchmarks::<S, String>("string", |val| format!("{val:020}"), &mut results);

    BenchmarkResult {
        version: BENCHMARK_RESULT_VERSION,
        results,
    }
}

fn run_type_benchmarks<S: Sort, T: Ord>(
    type_name: &str,
    type_producer: impl Fn(u64) -> T + Copy,
    results: &mut HashMap<BenchmarkResultKey, DurationOpaque>,
) {
    #[allow(clippy::type_complexity)]
    let pattern_providers: Vec<(&'static str, fn(usize) -> Vec<u64>)> = vec![
        // Tests worst case branch-misprediction, and stress tests the small-sorts.
        ("random", patterns::random),
        // Tests real-world random distributions. Mix of low-cardinality and plain random.
        ("random_z1", |len| patterns::random_zipf(len, 1.0)),
        // Tests low-cardinality with a low number of distinct values, often found in real world
        // data-sets.
        ("random_d20", |len| patterns::random_uniform(len, 0..20)),
        // Tests low-cardinality with one very common value, often found in real world data-sets.
        ("random_p5", |len| patterns::random_x_percent(len, 5.0)),
        // Tests append plus sort.
        ("random_s95", |len| patterns::random_sorted(len, 95.0)),
        // ascending and descending are omitted because they rely on code-alignment and run-time for
        // them can vary greatly even if nothing in the implementation changed.
    ];

    let test_lens = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 17, 20, 24, 35, 49, 70, 100, 200, 400, 900, 2_048,
        4_833, 10_000, 22_367, 50_000, 100_000, 183_845, 400_000, 1_000_000, 2_000_000, 4_281_332,
        10_000_000,
    ];

    let sort_name = S::name();

    for test_len in test_lens {
        for (pattern_name, pattern_provider) in &pattern_providers {
            if *pattern_name != "random" && test_len < 7 {
                continue;
            }

            // "cold" benchmarks in the sort-research-rs sense, where i-cache and btb are flushed
            // between measurement, are possible but difficult to reconcile with the goal of fast,
            // low-noise and as few false positives as possible.

            let median_duration = sample_duration::<S, T>(test_len, |len| {
                pattern_provider(len)
                    .into_iter()
                    .map(type_producer)
                    .collect()
            });

            results.insert(
                BenchmarkResultKey::new(format!(
                    "{sort_name}-hot-{type_name}-{pattern_name}-{test_len}"
                )),
                median_duration,
            );
        }
    }
}

fn sample_duration<S: Sort, T: Ord>(
    test_len: usize,
    pattern_provider: impl Fn(usize) -> Vec<T>,
) -> DurationOpaque {
    // Never does more than this many samples to start with.
    const MAX_INITIAL_SAMPLE_COUNT: usize = 5000;

    // Never does fewer than this many samples.
    const MIN_SAMPLE_COUNT: usize = 5;

    // With each re-sample, the sample-ratio increases by 4x, how many times is it allowed to
    // increase this ratio before giving up.
    const MAX_RE_SAMPLE_COUNT: usize = 5;

    // The maximum symmetric difference allowed in a 20% windows around the median value to allow
    // early exit sampling.
    const MAX_ALLOWED_DIFFERENCE: f64 = 1.02;

    let test_len_f64 = cmp::max(test_len, 1) as f64;
    let mut sample_count = cmp::max(
        cmp::min(
            (3e4 / ((test_len_f64 * 6e-4) * test_len_f64.log(1.4))).round() as usize,
            MAX_INITIAL_SAMPLE_COUNT,
        ),
        MIN_SAMPLE_COUNT,
    );
    let mut median_duration = None;

    for _ in 0..MAX_RE_SAMPLE_COUNT {
        //println!("test_len: {test_len} sample_count: {sample_count}");

        let warmup_sample_count = cmp::max((sample_count as f64 / 10.0).round() as usize, 1);

        let mut input_buffer = (0..(sample_count + warmup_sample_count))
            .map(|_| pattern_provider(test_len))
            .collect::<Vec<_>>();

        let mut durations = Vec::with_capacity(sample_count);

        for (i, input) in &mut input_buffer.iter_mut().enumerate() {
            let duration = measure_duration(|| S::sort(black_box(input)));
            black_box(input); // side-effect

            if i >= warmup_sample_count {
                durations.push(duration);
            }
        }

        let (variance, md) = DurationOpaque::analyze(&mut durations);

        if Some(md) == median_duration {
            // If we get the same median as before, it's not gonna change anymore for small size.
            break;
        }

        median_duration = Some(md);

        if variance <= MAX_ALLOWED_DIFFERENCE {
            break;
        }

        sample_count *= 2;
    }

    median_duration.unwrap()
}

pub fn pin_thread_to_core() {
    use std::cell::Cell;
    let pin_core_id: usize = 2;

    thread_local! {static AFFINITY_ALREADY_SET: Cell<bool> = const { Cell::new(false) } }

    // Set affinity only once per thread.
    AFFINITY_ALREADY_SET.with(|affinity_already_set| {
        if !affinity_already_set.get() {
            if let Some(core_id_2) = core_affinity::get_core_ids()
                .as_ref()
                .and_then(|ids| ids.get(pin_core_id))
            {
                core_affinity::set_for_current(*core_id_2);
            }

            affinity_already_set.set(true);
        }
    });
}

// 16 byte stack value, with more expensive comparison.
#[repr(C)]
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct F128 {
    x: f64,
    y: f64,
}

impl F128 {
    pub fn new(val: u64) -> Self {
        let val_f = val as f64;

        let x = val_f + 0.1;
        let y = val_f.log(4.1);

        Self { x, y }
    }
}

// This is kind of hacky, but we know we only have normal comparable floats in there.
impl Eq for F128 {}

impl PartialOrd for F128 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for F128 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Simulate expensive comparison function.
        let this_div = self.x / self.y;
        let other_div = other.x / other.y;

        // SAFETY: The constructor guarantees that the values are normal.
        unsafe { this_div.partial_cmp(&other_div).unwrap_unchecked() }
    }
}
