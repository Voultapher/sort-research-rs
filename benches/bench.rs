use std::cell::RefCell;
use std::collections::HashSet;
use std::env;
use std::rc::Rc;
use std::sync::Mutex;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[allow(unused_imports)]
use sort_test_tools::ffi_types::{FFIOneKiloByte, FFIString, F128};

use sort_test_tools::patterns;

#[allow(unused_imports)]
use sort_comp::{stable, unstable};

#[cfg(feature = "cold_benchmarks")]
mod trash_prediction;

mod bench_other;

use crate::bench_other::bench_other;
use crate::bench_other::util::bench_fn;

fn measure_comp_count(
    name: &str,
    test_size: usize,
    instrumented_sort_func: impl Fn(),
    comp_count: Rc<RefCell<u64>>,
) {
    // Measure how many comparisons are performed by a specific implementation and input
    // combination.
    let run_count: usize = if test_size <= 20 {
        100_000
    } else if test_size < 10_000 {
        3000
    } else if test_size < 100_000 {
        1000
    } else if test_size < 1_000_000 {
        100
    } else {
        10
    };

    *comp_count.borrow_mut() = 0;
    for _ in 0..run_count {
        instrumented_sort_func();
    }

    // If there is on average less than a single comparison this will be wrong.
    // But that's such a corner case I don't care about it.
    let total = *comp_count.borrow() / (run_count as u64);
    println!("{name}: mean comparisons: {total}");
}

#[inline(never)]
fn bench_impl<T: Ord + std::fmt::Debug, Sort: sort_test_tools::Sort>(
    c: &mut Criterion,
    test_size: usize,
    transform_name: &str,
    transform: &fn(Vec<i32>) -> Vec<T>,
    pattern_name: &str,
    pattern_provider: &fn(usize) -> Vec<i32>,
    _sort_impl: Sort,
) {
    let bench_name = Sort::name();

    if env::var("MEASURE_COMP").is_ok() {
        // Configure this to filter results. For now the only real difference is copy types.
        // if transform_name == "i32" && bench_name.contains("unstable") && test_size <= 100000 {
        if transform_name == "u64" && test_size >= 1_000_000 {
            // Abstracting over sort_by is kinda tricky without HKTs so a macro will do.
            let name = format!(
                "{}-comp-{}-{}-{}",
                bench_name, transform_name, pattern_name, test_size
            );

            // Instrument via sort_by to ensure the type properties such as Copy of the type
            // that is being sorted doesn't change. And we get representative numbers.
            let comp_count = Rc::new(RefCell::new(0u64));
            let comp_count_copy = comp_count.clone();
            let instrumented_sort_func = || {
                let mut test_data = transform(pattern_provider(test_size));
                Sort::sort_by(black_box(test_data.as_mut_slice()), |a, b| {
                    *comp_count_copy.borrow_mut() += 1;
                    a.cmp(b)
                })
            };
            measure_comp_count(&name, test_size, instrumented_sort_func, comp_count);
        }
    } else if env::var("BENCH_OTHER").is_ok() {
        static SEEN_BENCHMARKS: Mutex<Option<HashSet<String>>> = Mutex::new(None);

        let mut seen_benchmarks = SEEN_BENCHMARKS.lock().unwrap();

        if seen_benchmarks.is_none() {
            *seen_benchmarks = Some(HashSet::new());
        }

        let combination_name = format!("{transform_name}-{pattern_name}-{test_size}");
        let seen_before = !seen_benchmarks.as_mut().unwrap().insert(combination_name);

        // Other benchmarks will not use the sort functions so only call the other benchmark builder
        // once per pattern-type-len combination.
        if !seen_before {
            let args = env::args().collect::<Vec<_>>();
            // No clue how stable that is.
            let filter_arg = &args[args.len() - 2];

            bench_other(
                c,
                filter_arg,
                test_size,
                transform_name,
                transform,
                pattern_name,
                pattern_provider,
            );
        }
    } else {
        bench_fn(
            c,
            test_size,
            transform_name,
            transform,
            pattern_name,
            pattern_provider,
            &bench_name,
            Sort::sort,
        );
    }
}

fn shuffle_vec<T: Ord>(mut v: Vec<T>) -> Vec<T> {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    let mut rng = thread_rng();
    v.shuffle(&mut rng);

    v
}

fn split_len(len: usize, part_a_percent: f64) -> (usize, usize) {
    let len_a = ((len as f64 / 100.0) * part_a_percent).round() as usize;
    let len_b = len - len_a;

    (len_a, len_b)
}

// TODO move to patterns.
fn random_x_percent(len: usize, percent: f64) -> Vec<i32> {
    assert!(percent > 0.0 && percent < 100.0);

    let (len_zero, len_random_p) = split_len(len, 100.0 - percent);
    let v: Vec<i32> = std::iter::repeat(0)
        .take(len_zero)
        .chain(patterns::random(len_random_p))
        .collect();

    shuffle_vec(v)
}

fn bench_patterns<T: Ord + std::fmt::Debug>(
    c: &mut Criterion,
    test_size: usize,
    transform_name: &str,
    transform: fn(Vec<i32>) -> Vec<T>,
) {
    if test_size > 100_000 && !(transform_name == "i32" || transform_name == "u64") {
        // These are just too expensive.
        return;
    }

    let mut pattern_providers: Vec<(&'static str, fn(usize) -> Vec<i32>)> = vec![
        ("random", patterns::random),
        ("random_z1", |len| patterns::random_zipf(len, 1.0)),
        ("random_d20", |len| patterns::random_uniform(len, 0..20)),
        ("random_p5", |len| random_x_percent(len, 5.0)),
        ("random_s95", |len| patterns::random_sorted(len, 95.0)),
        ("ascending", patterns::ascending),
        ("descending", patterns::descending),
        ("saws_short", |len| patterns::saw_mixed_range(len, 20..70)),
    ];

    // Custom patterns designed to find worst case performance.
    let mut extra_pattern_providers: Vec<(&'static str, fn(usize) -> Vec<i32>)> = vec![
        ("saws_long", |len| {
            patterns::saw_mixed(len, ((len as f64).log2().round()) as usize)
        }),
        ("random_d20_start_block", |len| {
            let mut v = patterns::random_uniform(len, 0..20);
            let loop_end = std::cmp::min(len, 100);
            for i in 0..loop_end {
                v[i] = 0;
            }

            v
        }),
        ("90_one_10_zero", |len| {
            let (len_90, len_10) = split_len(len, 90.0);
            std::iter::repeat(1)
                .take(len_90)
                .chain(std::iter::repeat(0).take(len_10))
                .collect()
        }),
        ("90_zero_10_one", |len| {
            let (len_90, len_10) = split_len(len, 90.0);
            std::iter::repeat(0)
                .take(len_90)
                .chain(std::iter::repeat(1).take(len_10))
                .collect()
        }),
        ("90_zero_10_random", |len| {
            let (len_90, len_10) = split_len(len, 90.0);
            std::iter::repeat(0)
                .take(len_90)
                .chain(patterns::random(len_10))
                .collect()
        }),
        ("90p_zero_10p_one", |len| {
            let (len_90p, len_10p) = split_len(len, 90.0);
            let v: Vec<i32> = std::iter::repeat(0)
                .take(len_90p)
                .chain(std::iter::repeat(1).take(len_10p))
                .collect();

            shuffle_vec(v)
        }),
        ("90p_zero_10p_random_dense_neg", |len| {
            let (len_90p, len_10p) = split_len(len, 90.0);
            let v: Vec<i32> = std::iter::repeat(0)
                .take(len_90p)
                .chain(patterns::random_uniform(len_10p, -10..=10))
                .collect();

            shuffle_vec(v)
        }),
        ("90p_zero_10p_random_dense_pos", |len| {
            let (len_90p, len_10p) = split_len(len, 90.0);
            let v: Vec<i32> = std::iter::repeat(0)
                .take(len_90p)
                .chain(patterns::random_uniform(len_10p, 0..=10))
                .collect();

            shuffle_vec(v)
        }),
        ("90p_zero_10p_random", |len| {
            let (len_90p, len_10p) = split_len(len, 90.0);
            let v: Vec<i32> = std::iter::repeat(0)
                .take(len_90p)
                .chain(patterns::random(len_10p))
                .collect();

            shuffle_vec(v)
        }),
        ("95p_zero_5p_random", |len| {
            let (len_95p, len_5p) = split_len(len, 95.0);
            let v: Vec<i32> = std::iter::repeat(0)
                .take(len_95p)
                .chain(patterns::random(len_5p))
                .collect();

            shuffle_vec(v)
        }),
        ("99p_zero_1p_random", |len| {
            let (len_99p, len_1p) = split_len(len, 99.0);
            let v: Vec<i32> = std::iter::repeat(0)
                .take(len_99p)
                .chain(patterns::random(len_1p))
                .collect();

            shuffle_vec(v)
        }),
        ("ascending_saw", |len| {
            patterns::ascending_saw(len, ((len as f64).log2().round()) as usize)
        }),
        ("descending_saw", |len| {
            patterns::descending_saw(len, ((len as f64).log2().round()) as usize)
        }),
        ("pipe_organ", patterns::pipe_organ),
        ("random__div3", |len| {
            patterns::random_uniform(len, 0..=(((len as f64 / 3.0).round()) as i32))
        }),
        ("random__div5", |len| {
            patterns::random_uniform(len, 0..=(((len as f64 / 3.0).round()) as i32))
        }),
        ("random__div8", |len| {
            patterns::random_uniform(len, 0..=(((len as f64 / 3.0).round()) as i32))
        }),
        ("random_d2", |len| patterns::random_uniform(len, 0..2)),
        ("random_d3", |len| patterns::random_uniform(len, 0..3)),
        ("random_d4", |len| patterns::random_uniform(len, 0..4)),
        ("random_d8", |len| patterns::random_uniform(len, 0..8)),
        ("random_d10", |len| patterns::random_uniform(len, 0..10)),
        ("random_d16", |len| patterns::random_uniform(len, 0..16)),
        ("random_d32", |len| patterns::random_uniform(len, 0..32)),
        ("random_d64", |len| patterns::random_uniform(len, 0..64)),
        ("random_d128", |len| patterns::random_uniform(len, 0..128)),
        ("random_d256", |len| patterns::random_uniform(len, 0..256)),
        ("random_d512", |len| patterns::random_uniform(len, 0..512)),
        ("random_d1024", |len| patterns::random_uniform(len, 0..1024)),
        ("random_p1", |len| random_x_percent(len, 1.0)),
        ("random_p2", |len| random_x_percent(len, 2.0)),
        ("random_p4", |len| random_x_percent(len, 4.0)),
        ("random_p6", |len| random_x_percent(len, 6.0)),
        ("random_p8", |len| random_x_percent(len, 8.0)),
        ("random_p10", |len| random_x_percent(len, 10.0)),
        ("random_p15", |len| random_x_percent(len, 15.0)),
        ("random_p20", |len| random_x_percent(len, 20.0)),
        ("random_p30", |len| random_x_percent(len, 30.0)),
        ("random_p40", |len| random_x_percent(len, 40.0)),
        ("random_p50", |len| random_x_percent(len, 50.0)),
        ("random_p60", |len| random_x_percent(len, 60.0)),
        ("random_p70", |len| random_x_percent(len, 70.0)),
        ("random_p80", |len| random_x_percent(len, 80.0)),
        ("random_p90", |len| random_x_percent(len, 90.0)),
        ("random_p95", |len| random_x_percent(len, 95.0)),
        ("random_p99", |len| random_x_percent(len, 99.0)),
        ("random_z1_05", |len| patterns::random_zipf(len, 1.05)),
        ("random_z1_1", |len| patterns::random_zipf(len, 1.1)),
        ("random_z1_2", |len| patterns::random_zipf(len, 1.2)),
        ("random_z1_3", |len| patterns::random_zipf(len, 1.3)),
        ("random_z1_4", |len| patterns::random_zipf(len, 1.4)),
        ("random_z1_6", |len| patterns::random_zipf(len, 1.6)),
        ("random_z2", |len| patterns::random_zipf(len, 2.0)),
        ("random_z3", |len| patterns::random_zipf(len, 3.0)),
        ("random_z4", |len| patterns::random_zipf(len, 4.0)),
        ("random_s5", |len| patterns::random_sorted(len, 95.0)),
        ("random_s5", |len| patterns::random_sorted(len, 5.0)),
        ("random_s10", |len| patterns::random_sorted(len, 10.0)),
        ("random_s30", |len| patterns::random_sorted(len, 30.0)),
        ("random_s50", |len| patterns::random_sorted(len, 50.0)),
        ("random_s70", |len| patterns::random_sorted(len, 70.0)),
        ("random_s90", |len| patterns::random_sorted(len, 90.0)),
        ("random_s99", |len| patterns::random_sorted(len, 99.0)),
    ];

    if env::var("EXTRA_PATTERNS").is_ok() {
        pattern_providers.append(&mut extra_pattern_providers);
    }

    for (pattern_name, pattern_provider) in pattern_providers.iter() {
        if test_size < 3 && *pattern_name != "random" {
            continue;
        }

        // --- Stable sorts ---

        // bench_impl(
        //     c,
        //     test_size,
        //     transform_name,
        //     &transform,
        //     pattern_name,
        //     pattern_provider,
        //     stable::rust_ipn::SortImpl,
        // );

        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            stable::rust_std::SortImpl,
        );

        #[cfg(feature = "cpp_std_sys")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            stable::cpp_std_sys::SortImpl,
        );

        #[cfg(feature = "cpp_std_libcxx")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            stable::cpp_std_libcxx::SortImpl,
        );

        #[cfg(feature = "cpp_std_gcc4_3")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            stable::cpp_std_gcc4_3::SortImpl,
        );

        #[cfg(feature = "cpp_powersort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            stable::cpp_powersort::SortImpl,
        );

        #[cfg(feature = "cpp_powersort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            stable::cpp_powersort_4way::SortImpl,
        );

        #[cfg(feature = "c_fluxsort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            stable::c_fluxsort::SortImpl,
        );

        #[cfg(feature = "rust_wpwoodjr")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            stable::rust_wpwoodjr::SortImpl,
        );

        #[cfg(feature = "rust_glidesort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            stable::rust_glidesort::SortImpl,
        );

        // --- Unstable sorts ---

        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            unstable::rust_ipnsort::SortImpl,
        );

        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            unstable::rust_std::SortImpl,
        );

        #[cfg(feature = "rust_dmsort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            unstable::rust_dmsort::SortImpl,
        );

        #[cfg(feature = "rust_crumsort_rs")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            unstable::rust_crumsort_rs::SortImpl,
        );

        #[cfg(feature = "cpp_pdqsort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            unstable::cpp_pdqsort::SortImpl,
        );

        #[cfg(feature = "cpp_ips4o")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            unstable::cpp_ips4o::SortImpl,
        );

        #[cfg(feature = "cpp_blockquicksort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            unstable::cpp_blockquicksort::SortImpl,
        );

        #[cfg(feature = "cpp_gerbens_qsort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            unstable::cpp_gerbens_qsort::SortImpl,
        );

        #[cfg(feature = "c_crumsort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            unstable::c_crumsort::SortImpl,
        );

        #[cfg(feature = "cpp_std_sys")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            unstable::cpp_std_sys::SortImpl,
        );

        #[cfg(feature = "cpp_std_libcxx")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            unstable::cpp_std_libcxx::SortImpl,
        );

        #[cfg(feature = "cpp_std_gcc4_3")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            unstable::cpp_std_gcc4_3::SortImpl,
        );

        // --- Other sorts ---

        #[cfg(feature = "rust_radsort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            sort_comp::other::rust_radsort::SortImpl,
        );

        #[cfg(feature = "cpp_simdsort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            sort_comp::other::cpp_simdsort::SortImpl,
        );

        #[cfg(feature = "cpp_vqsort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            sort_comp::other::cpp_vqsort::SortImpl,
        );

        #[cfg(feature = "cpp_intel_avx512")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            sort_comp::other::cpp_intel_avx512::SortImpl,
        );

        // --- Evolution ---

        #[cfg(feature = "evolution")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            sort_comp::other::sort_evolution::stable::timsort_evo0::SortImpl,
        );

        #[cfg(feature = "evolution")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            sort_comp::other::sort_evolution::stable::timsort_evo1::SortImpl,
        );

        #[cfg(feature = "evolution")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            sort_comp::other::sort_evolution::stable::timsort_evo2::SortImpl,
        );

        #[cfg(feature = "evolution")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            sort_comp::other::sort_evolution::stable::timsort_evo3::SortImpl,
        );

        #[cfg(feature = "evolution")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            sort_comp::other::sort_evolution::stable::timsort_evo4::SortImpl,
        );

        #[cfg(feature = "small_sort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            sort_comp::other::small_sort::sort4_unstable_cmp_swap::SortImpl,
        );

        #[cfg(feature = "small_sort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            sort_comp::other::small_sort::sort4_unstable_ptr_select::SortImpl,
        );

        #[cfg(feature = "small_sort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            sort_comp::other::small_sort::sort4_unstable_branchy::SortImpl,
        );

        #[cfg(feature = "small_sort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            sort_comp::other::small_sort::sort4_stable_orson::SortImpl,
        );

        #[cfg(feature = "small_sort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            sort_comp::other::small_sort::sort10_unstable_cmp_swaps::SortImpl,
        );
        #[cfg(feature = "small_sort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            sort_comp::other::small_sort::sort10_unstable_experimental::SortImpl,
        );
        #[cfg(feature = "small_sort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            sort_comp::other::small_sort::sort10_unstable_ptr_select::SortImpl,
        );
    }
}

fn ensure_true_random() {
    // Ensure that random vecs are actually different.
    let random_vec_a = patterns::random(5);
    let random_vec_b = patterns::random(5);

    // I had a bug, where the test logic for fixed seeds, made the benchmarks always use the same
    // numbers, and random wasn't random at all anymore.
    assert_ne!(random_vec_a, random_vec_b);
}

fn criterion_benchmark(c: &mut Criterion) {
    // Distribute points somewhat evenly up to 1e7 in log10 space.
    let test_sizes = [
        0, 1, 2, 3, 4, 6, 8, 10, 12, 17, 24, 35, 49, 70, 100, 200, 400, 900, 2_048, 4_833, 10_000,
        22_367, 50_000, 100_000, 183_845, 400_000, 1_000_000, 2_000_000, 4_281_332, 10_000_000,
    ];

    patterns::disable_fixed_seed();
    ensure_true_random();

    for test_size in test_sizes {
        // Basic type often used to test sorting algorithms.
        bench_patterns(c, test_size, "i32", |values| values);

        // Common type for usize on 64-bit machines.
        // Sorting indices is very common.
        bench_patterns(c, test_size, "u64", |values| {
            values
                .iter()
                .map(|val| -> u64 {
                    // Extends the value into the 64 bit range,
                    // while preserving input order.
                    let x = ((*val as i64) + (i32::MAX as i64) + 1) as u64;
                    x.checked_mul(i32::MAX as u64).unwrap()
                })
                .collect()
        });

        // bench_patterns(c, test_size, "rust_string", |values| {
        //     // Strings are compared lexicographically, so we zero extend them to maintain the input
        //     // order.
        //     // See: https://godbolt.org/z/M38zTK6nv and https://godbolt.org/z/G18Yb7zoE
        //     values
        //         .iter()
        //         .map(|val| format!("{:010}", val.saturating_abs()))
        //         .collect()
        // });

        // Larger type that is not Copy and does heap access.
        // FFI String
        bench_patterns(c, test_size, "string", |values| {
            values
                .iter()
                .map(|val| FFIString::new(format!("{:010}", val.saturating_abs())))
                .collect()
        });

        // Very large stack value.
        bench_patterns(c, test_size, "1k", |values| {
            values.iter().map(|val| FFIOneKiloByte::new(*val)).collect()
        });

        // 16 byte stack value that is Copy but has a relatively expensive cmp implementation.
        bench_patterns(c, test_size, "f128", |values| {
            values.iter().map(|val| F128::new(*val)).collect()
        });

        // use std::cmp::Ordering;
        // use std::sync::Mutex;

        // #[derive(Debug)]
        // struct ValWithMutex {
        //     val: u64,
        //     mutex: Mutex<u64>,
        // }

        // impl PartialEq for ValWithMutex {
        //     fn eq(&self, other: &Self) -> bool {
        //         self.val == other.val
        //     }
        // }

        // impl Eq for ValWithMutex {}

        // impl PartialOrd for ValWithMutex {
        //     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        //         self.val.partial_cmp(&other.val)
        //     }
        // }

        // impl Ord for ValWithMutex {
        //     fn cmp(&self, other: &Self) -> Ordering {
        //         self.partial_cmp(other).unwrap()
        //     }
        // }

        // bench_patterns(c, test_size, "val_with_mutex", |values| {
        //     values
        //         .iter()
        //         .map(|val| -> ValWithMutex {
        //             let mut val_u64 = ((*val as i64) + (i32::MAX as i64) + 1) as u64;
        //             val_u64 = val_u64.checked_mul(i32::MAX as u64).unwrap();

        //             let mut this = ValWithMutex {
        //                 val: val_u64,
        //                 mutex: Mutex::new(val_u64),
        //             };

        //             // To make sure mutex is not optimized away.
        //             this.val = *this.mutex.lock().unwrap();

        //             this
        //         })
        //         .collect()
        // });

        // bench_patterns(c, test_size, "u8", |values| {
        //     values
        //         .iter()
        //         .map(|val| -> u8 { (val & u8::MAX as i32) as u8 })
        //         .collect()
        // });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
