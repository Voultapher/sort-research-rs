#![feature(local_key_cell_methods)]

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use regex::Regex;

use once_cell::sync::OnceCell;

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

#[allow(unused_imports)]
use sort_test_tools::ffi_types::{FFIOneKiloByte, FFIString, F128};

use sort_test_tools::patterns;

#[allow(unused_imports)]
use sort_comp::{stable, unstable};

#[cfg(feature = "cold_benchmarks")]
mod trash_prediction;

mod bench_custom;
mod util;

use crate::bench_custom::bench_custom;
use crate::util::pin_thread_to_core;

#[inline(never)]
fn bench_sort<T: Ord + std::fmt::Debug>(
    c: &mut Criterion,
    test_size: usize,
    transform_name: &str,
    transform: &fn(Vec<i32>) -> Vec<T>,
    pattern_name: &str,
    pattern_provider: &fn(usize) -> Vec<i32>,
    mut bench_name: &str,
    sort_func: impl Fn(&mut [T]),
) {
    // Pin the benchmark to the same core to improve repeatability. Doing it this way allows
    // criterion to do other stuff with other threads, which greatly impacts overall benchmark
    // throughput.
    pin_thread_to_core();

    let batch_size = if test_size > 30 {
        BatchSize::LargeInput
    } else {
        BatchSize::SmallInput
    };

    static FILTER_REGEX: OnceCell<Option<regex::Regex>> = OnceCell::new();

    let is_bench_name_ok = |name: &str| -> bool {
        let filter_regex = FILTER_REGEX.get_or_init(|| {
            env::var("CUSTOM_BENCH_REGEX")
                .ok()
                .map(|filter_regex| Regex::new(&filter_regex).unwrap())
        });

        filter_regex
            .as_ref()
            .map(|reg| reg.is_match(name))
            .unwrap_or(true)
    };

    static NAME_OVERWRITE: OnceCell<Option<String>> = OnceCell::new();

    let name_overwrite = NAME_OVERWRITE.get_or_init(|| env::var("BENCH_NAME_OVERWRITE").ok());

    if let Some(name) = name_overwrite {
        let split_pos = name.find(":").unwrap();
        let match_name = &name[..split_pos];
        if bench_name != match_name {
            return;
        }

        bench_name = &name[(split_pos + 1)..];
    }

    let bench_name_hot = format!("{bench_name}-hot-{transform_name}-{pattern_name}-{test_size}");
    if is_bench_name_ok(&bench_name_hot) {
        c.bench_function(&bench_name_hot, |b| {
            b.iter_batched(
                || transform(pattern_provider(test_size)),
                |mut test_data| {
                    sort_func(black_box(test_data.as_mut_slice()));
                    black_box(test_data); // side-effect
                },
                batch_size,
            )
        });
    }

    #[cfg(feature = "cold_benchmarks")]
    {
        let bench_name_cold =
            format!("{bench_name}-cold-{transform_name}-{pattern_name}-{test_size}");
        if is_bench_name_ok(&bench_name_cold) {
            c.bench_function(&bench_name_cold, |b| {
                b.iter_batched(
                    || {
                        let mut test_ints = pattern_provider(test_size);

                        if test_ints.len() == 0 {
                            return vec![];
                        }

                        // Try as best as possible to trash all prediction state in the CPU, to
                        // simulate calling the benchmark function as part of a larger program.
                        // Caveat, memory caches. We don't want to benchmark how expensive it is to
                        // load something from main memory.
                        let first_val =
                            black_box(trash_prediction::trash_prediction_state(test_ints[0]));

                        // Limit the optimizer in getting rid of trash_prediction_state,
                        // by tying its output to the test input.
                        test_ints[0] = first_val;

                        transform(test_ints)
                    },
                    |mut test_data| {
                        sort_func(black_box(test_data.as_mut_slice()));
                        black_box(test_data); // side-effect
                    },
                    BatchSize::PerIteration,
                )
            });
        }
    }
}

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
    } else {
        100
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
        if transform_name == "i32" && bench_name.contains("unstable") && test_size <= 100000 {
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
    } else if env::var("BENCH_CUSTOM").is_ok() {
        let args = env::args().collect::<Vec<_>>();
        // No clue how stable that is.
        let filter_arg = &args[args.len() - 2];

        bench_custom(
            filter_arg,
            test_size,
            transform_name,
            transform,
            pattern_name,
            pattern_provider,
        );
    } else {
        bench_sort(
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

fn split_len(size: usize, part_a_percent: f64) -> (usize, usize) {
    let len_a = ((size as f64 / 100.0) * part_a_percent).round() as usize;
    let len_b = size - len_a;

    (len_a, len_b)
}

// TODO move to patterns.
fn random_x_percent(size: usize, percent: f64) -> Vec<i32> {
    assert!(percent > 0.0 && percent < 100.0);

    let (len_zero, len_random_p) = split_len(size, 100.0 - percent);
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
        ("random_z1", |size| patterns::random_zipf(size, 1.0)),
        ("random_d20", |size| patterns::random_uniform(size, 0..20)),
        ("random_p5", |size| random_x_percent(size, 5.0)),
        ("random_s95", |size| patterns::random_sorted(size, 95.0)),
        ("ascending", patterns::ascending),
        ("descending", patterns::descending),
        ("saws_short", |size| patterns::saw_mixed_range(size, 20..70)),
    ];

    // Custom patterns designed to find worst case performance.
    let mut extra_pattern_providers: Vec<(&'static str, fn(usize) -> Vec<i32>)> = vec![
        ("saws_long", |size| {
            patterns::saw_mixed(size, ((size as f64).log2().round()) as usize)
        }),
        ("random_d20_start_block", |size| {
            let mut v = patterns::random_uniform(size, 0..20);
            let loop_end = std::cmp::min(size, 100);
            for i in 0..loop_end {
                v[i] = 0;
            }

            v
        }),
        ("90_one_10_zero", |size| {
            let (len_90, len_10) = split_len(size, 90.0);
            std::iter::repeat(1)
                .take(len_90)
                .chain(std::iter::repeat(0).take(len_10))
                .collect()
        }),
        ("90_zero_10_one", |size| {
            let (len_90, len_10) = split_len(size, 90.0);
            std::iter::repeat(0)
                .take(len_90)
                .chain(std::iter::repeat(1).take(len_10))
                .collect()
        }),
        ("90_zero_10_random", |size| {
            let (len_90, len_10) = split_len(size, 90.0);
            std::iter::repeat(0)
                .take(len_90)
                .chain(patterns::random(len_10))
                .collect()
        }),
        ("90p_zero_10p_one", |size| {
            let (len_90p, len_10p) = split_len(size, 90.0);
            let v: Vec<i32> = std::iter::repeat(0)
                .take(len_90p)
                .chain(std::iter::repeat(1).take(len_10p))
                .collect();

            shuffle_vec(v)
        }),
        ("90p_zero_10p_random_dense_neg", |size| {
            let (len_90p, len_10p) = split_len(size, 90.0);
            let v: Vec<i32> = std::iter::repeat(0)
                .take(len_90p)
                .chain(patterns::random_uniform(len_10p, -10..=10))
                .collect();

            shuffle_vec(v)
        }),
        ("90p_zero_10p_random_dense_pos", |size| {
            let (len_90p, len_10p) = split_len(size, 90.0);
            let v: Vec<i32> = std::iter::repeat(0)
                .take(len_90p)
                .chain(patterns::random_uniform(len_10p, 0..=10))
                .collect();

            shuffle_vec(v)
        }),
        ("90p_zero_10p_random", |size| {
            let (len_90p, len_10p) = split_len(size, 90.0);
            let v: Vec<i32> = std::iter::repeat(0)
                .take(len_90p)
                .chain(patterns::random(len_10p))
                .collect();

            shuffle_vec(v)
        }),
        ("95p_zero_5p_random", |size| {
            let (len_95p, len_5p) = split_len(size, 95.0);
            let v: Vec<i32> = std::iter::repeat(0)
                .take(len_95p)
                .chain(patterns::random(len_5p))
                .collect();

            shuffle_vec(v)
        }),
        ("99p_zero_1p_random", |size| {
            let (len_99p, len_1p) = split_len(size, 99.0);
            let v: Vec<i32> = std::iter::repeat(0)
                .take(len_99p)
                .chain(patterns::random(len_1p))
                .collect();

            shuffle_vec(v)
        }),
        ("ascending_saw", |size| {
            patterns::ascending_saw(size, ((size as f64).log2().round()) as usize)
        }),
        ("descending_saw", |size| {
            patterns::descending_saw(size, ((size as f64).log2().round()) as usize)
        }),
        ("pipe_organ", patterns::pipe_organ),
        ("random__div3", |size| {
            patterns::random_uniform(size, 0..=(((size as f64 / 3.0).round()) as i32))
        }),
        ("random__div5", |size| {
            patterns::random_uniform(size, 0..=(((size as f64 / 3.0).round()) as i32))
        }),
        ("random__div8", |size| {
            patterns::random_uniform(size, 0..=(((size as f64 / 3.0).round()) as i32))
        }),
        ("random_d2", |size| patterns::random_uniform(size, 0..2)),
        ("random_d4", |size| patterns::random_uniform(size, 0..4)),
        ("random_d8", |size| patterns::random_uniform(size, 0..8)),
        ("random_d10", |size| patterns::random_uniform(size, 0..10)),
        ("random_d16", |size| patterns::random_uniform(size, 0..16)),
        ("random_d32", |size| patterns::random_uniform(size, 0..32)),
        ("random_d64", |size| patterns::random_uniform(size, 0..64)),
        ("random_d128", |size| patterns::random_uniform(size, 0..128)),
        ("random_d256", |size| patterns::random_uniform(size, 0..256)),
        ("random_d512", |size| patterns::random_uniform(size, 0..512)),
        ("random_d1024", |size| {
            patterns::random_uniform(size, 0..1024)
        }),
        ("random_p1", |size| random_x_percent(size, 1.0)),
        ("random_p2", |size| random_x_percent(size, 2.0)),
        ("random_p4", |size| random_x_percent(size, 4.0)),
        ("random_p6", |size| random_x_percent(size, 6.0)),
        ("random_p8", |size| random_x_percent(size, 8.0)),
        ("random_p10", |size| random_x_percent(size, 10.0)),
        ("random_p15", |size| random_x_percent(size, 15.0)),
        ("random_p20", |size| random_x_percent(size, 20.0)),
        ("random_p30", |size| random_x_percent(size, 30.0)),
        ("random_p40", |size| random_x_percent(size, 40.0)),
        ("random_p50", |size| random_x_percent(size, 50.0)),
        ("random_p60", |size| random_x_percent(size, 60.0)),
        ("random_p70", |size| random_x_percent(size, 70.0)),
        ("random_p80", |size| random_x_percent(size, 80.0)),
        ("random_p90", |size| random_x_percent(size, 90.0)),
        ("random_p95", |size| random_x_percent(size, 95.0)),
        ("random_p99", |size| random_x_percent(size, 99.0)),
        ("random_z1_05", |size| patterns::random_zipf(size, 1.05)),
        ("random_z1_1", |size| patterns::random_zipf(size, 1.1)),
        ("random_z1_2", |size| patterns::random_zipf(size, 1.2)),
        ("random_z1_3", |size| patterns::random_zipf(size, 1.3)),
        ("random_z1_4", |size| patterns::random_zipf(size, 1.4)),
        ("random_z1_6", |size| patterns::random_zipf(size, 1.6)),
        ("random_z2", |size| patterns::random_zipf(size, 2.0)),
        ("random_z3", |size| patterns::random_zipf(size, 3.0)),
        ("random_z4", |size| patterns::random_zipf(size, 4.0)),
        ("random_s5", |size| patterns::random_sorted(size, 95.0)),
        ("random_s5", |size| patterns::random_sorted(size, 5.0)),
        ("random_s10", |size| patterns::random_sorted(size, 10.0)),
        ("random_s30", |size| patterns::random_sorted(size, 30.0)),
        ("random_s50", |size| patterns::random_sorted(size, 50.0)),
        ("random_s70", |size| patterns::random_sorted(size, 70.0)),
        ("random_s90", |size| patterns::random_sorted(size, 90.0)),
        ("random_s99", |size| patterns::random_sorted(size, 99.0)),
    ];

    if env::var("EXTRA_PATTERNS").is_ok() {
        pattern_providers.append(&mut extra_pattern_providers);
    }

    for (pattern_name, pattern_provider) in pattern_providers.iter() {
        if test_size < 3 && *pattern_name != "random" {
            continue;
        }

        // --- Stable sorts ---

        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            stable::rust_ipn::SortImpl,
        );

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
            unstable::rust_ipn::SortImpl,
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
        0, 1, 2, 3, 4, 6, 8, 12, 17, 24, 35, 49, 70, 100, 200, 400, 900, 2_048, 4_833, 10_000,
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
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
