#![feature(local_key_cell_methods)]

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

#[allow(unused_imports)]
use sort_comp::{
    ffi_util::FFIOneKiloByte, ffi_util::FFIString, ffi_util::F128, patterns, stable, unstable,
};

mod trash_prediction;
use trash_prediction::trash_prediction_state;

mod bench_custom;
use bench_custom::bench_custom;

fn pin_thread_to_core() {
    use std::cell::Cell;
    let pin_core_id: usize = 2;

    thread_local! {static AFFINITY_ALREADY_SET: Cell<bool> = Cell::new(false); }

    // Set affinity only once per thread.
    if !AFFINITY_ALREADY_SET.get() {
        if let Some(core_id_2) = core_affinity::get_core_ids()
            .as_ref()
            .and_then(|ids| ids.get(pin_core_id))
        {
            core_affinity::set_for_current(*core_id_2);
        }

        AFFINITY_ALREADY_SET.set(true);
    }
}

#[inline(never)]
fn bench_sort<T: Ord + std::fmt::Debug>(
    c: &mut Criterion,
    test_size: usize,
    transform_name: &str,
    transform: &fn(Vec<i32>) -> Vec<T>,
    pattern_name: &str,
    pattern_provider: &fn(usize) -> Vec<i32>,
    bench_name: &str,
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

    c.bench_function(
        &format!("{bench_name}-hot-{transform_name}-{pattern_name}-{test_size}"),
        |b| {
            b.iter_batched(
                || transform(pattern_provider(test_size)),
                |mut test_data| sort_func(black_box(test_data.as_mut_slice())),
                batch_size,
            )
        },
    );

    c.bench_function(
        &format!("{bench_name}-cold-{transform_name}-{pattern_name}-{test_size}"),
        |b| {
            b.iter_batched(
                || {
                    let mut test_ints = pattern_provider(test_size);

                    if test_ints.len() == 0 {
                        return vec![];
                    }

                    // Try as best as possible to trash all prediction state in the CPU, to simulate
                    // calling the benchmark function as part of a larger program. Caveat, memory
                    // caches. We don't want to benchmark how expensive it is to load something from
                    // main memory.
                    let first_val = black_box(trash_prediction_state(test_ints[0]));

                    // Limit the optimizer in getting rid of trash_prediction_state,
                    // by tying its output to the test input.
                    test_ints[0] = first_val;

                    transform(test_ints)
                },
                |mut test_data| sort_func(black_box(test_data.as_mut_slice())),
                BatchSize::PerIteration,
            )
        },
    );
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
fn bench_impl<T: Ord + std::fmt::Debug, Sort: sort_comp::Sort>(
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
        ("random_dense", |size| {
            patterns::random_uniform(size, 0..=(((size as f64).log2().round()) as i32) as i32)
        }),
        ("random_binary", |size| {
            patterns::random_uniform(size, 0..=1 as i32)
        }),
        ("random_5p", |size| {
            let (len_95p, len_5p) = split_len(size, 95.0);
            let v: Vec<i32> = std::iter::repeat(0)
                .take(len_95p)
                .chain(patterns::random(len_5p))
                .collect();

            shuffle_vec(v)
        }),
        ("ascending", patterns::ascending),
        ("descending", patterns::descending),
        ("saws_long", |size| {
            patterns::saw_mixed(size, ((size as f64).log2().round()) as usize)
        }),
        ("saws_short", |size| {
            patterns::saw_mixed(size, (size as f64 / 22.0).round() as usize)
        }),
    ];

    // Custom patterns designed to find worst case performance.
    let mut extra_pattern_providers: Vec<(&'static str, fn(usize) -> Vec<i32>)> = vec![
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
        ("random_div3", |size| {
            patterns::random_uniform(size, 0..=(((size as f64 / 3.0).round()) as i32))
        }),
        ("random_div5", |size| {
            patterns::random_uniform(size, 0..=(((size as f64 / 3.0).round()) as i32))
        }),
        ("random_div8", |size| {
            patterns::random_uniform(size, 0..=(((size as f64 / 3.0).round()) as i32))
        }),
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

        #[cfg(feature = "cpp_highwaysort")]
        bench_impl(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            sort_comp::other::cpp_highwaysort::SortImpl,
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
    let test_sizes = [
        0, 1, 2, 3, 5, 7, 8, 9, 11, 13, 15, 16, 17, 19, 20, 24, 28, 31, 36, 50, 101, 200, 500,
        1_000, 2_048, 10_000, 100_000, 1_000_000, 10_000_000,
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
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
