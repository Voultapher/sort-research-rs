use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

#[allow(unused_imports)]
use sort_comp::{patterns, stable, unstable};

mod trash_prediction;
use trash_prediction::trash_prediction_state;

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
    } else if test_size < 1_000_000 {
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

macro_rules! bench_func {
    (
        $c:expr,
        $test_size:expr,
        $transform_name:expr,
        $transform:expr,
        $pattern_name:expr,
        $pattern_provider:expr,
        $bench_name:ident,
        $bench_module:path,
    ) => {
        use $bench_module as $bench_name;
        let bench_name = stringify!($bench_name);

        if env::var("MEASURE_COMP").is_ok() {
            // Configure this to filter results. For now the only real difference is copy types.
            if $transform_name == "i32"
                // && $test_size <= 100000
                && $pattern_name != &"random_random_size"
            {
                // Abstracting over sort_by is kinda tricky without HKTs so a macro will do.
                let name = format!(
                    "{}-comp-{}-{}-{}",
                    bench_name, $transform_name, $pattern_name, $test_size
                );
                // Instrument via sort_by to ensure the type properties such as Copy of the type
                // that is being sorted doesn't change. And we get representative numbers.
                let comp_count = Rc::new(RefCell::new(0u64));
                let comp_count_copy = comp_count.clone();
                let instrumented_sort_func = || {
                    let mut test_data = $transform($pattern_provider($test_size));
                    $bench_name::sort_by(black_box(test_data.as_mut_slice()), |a, b| {
                        *comp_count_copy.borrow_mut() += 1;
                        a.cmp(b)
                    })
                };
                measure_comp_count(&name, $test_size, instrumented_sort_func, comp_count);
            }
        } else {
            bench_sort(
                $c,
                $test_size,
                $transform_name,
                $transform,
                $pattern_name,
                $pattern_provider,
                bench_name,
                $bench_name::sort,
            );
        }
    };
}

fn bench_patterns<T: Ord + std::fmt::Debug + Clone>(
    c: &mut Criterion,
    test_size: usize,
    transform_name: &str,
    transform: fn(Vec<i32>) -> Vec<T>,
) {
    if test_size > 100_000 && !(transform_name == "i32" || transform_name == "u64") {
        // These are just too expensive.
        return;
    }

    let pattern_providers: Vec<(&'static str, fn(usize) -> Vec<i32>)> = vec![
        ("random", patterns::random),
        ("random_dense", |size| {
            patterns::random_uniform(size, 0..(((size as f64).log2().round()) as i32) as i32)
        }),
        ("random_binary", |size| {
            patterns::random_uniform(size, 0..1 as i32)
        }),
        ("ascending", patterns::ascending),
        ("descending", patterns::descending),
        ("ascending_saw", |size| {
            patterns::ascending_saw(size, ((size as f64).log2().round()) as usize)
        }),
        ("descending_saw", |size| {
            patterns::descending_saw(size, ((size as f64).log2().round()) as usize)
        }),
        ("pipe_organ", patterns::pipe_organ),
    ];

    for (pattern_name, pattern_provider) in pattern_providers.iter() {
        if test_size < 3 && *pattern_name != "random" {
            continue;
        }

        // --- Stable sorts ---

        bench_func!(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            rust_new_stable,
            stable::rust_new,
        );

        bench_func!(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            rust_std_stable,
            stable::rust_std,
        );

        #[cfg(feature = "cpp_std_sys")]
        bench_func!(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            cpp_std_sys_stable,
            stable::cpp_std_sys,
        );

        #[cfg(feature = "cpp_std_libcxx")]
        bench_func!(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            cpp_std_libcxx_stable,
            stable::cpp_std_libcxx,
        );

        #[cfg(feature = "cpp_powersort")]
        bench_func!(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            cpp_powersort_stable,
            stable::cpp_powersort,
        );

        #[cfg(feature = "cpp_powersort")]
        bench_func!(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            cpp_powersort_4way_stable,
            stable::cpp_powersort_4way,
        );

        #[cfg(feature = "c_fluxsort")]
        bench_func!(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            c_fluxsort_stable,
            stable::c_fluxsort,
        );

        #[cfg(feature = "rust_wpwoodjr")]
        bench_func!(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            rust_wpwoodjr_stable,
            stable::rust_wpwoodjr,
        );

        // --- Unstable sorts ---

        bench_func!(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            rust_new_unstable,
            unstable::rust_new,
        );

        bench_func!(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            rust_std_unstable,
            unstable::rust_std,
        );

        #[cfg(feature = "rust_dmsort")]
        bench_func!(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            rust_dmsort_unstable,
            unstable::rust_dmsort,
        );

        #[cfg(feature = "cpp_pdqsort")]
        bench_func!(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            cpp_pdq_unstable,
            unstable::cpp_pdqsort,
        );

        #[cfg(feature = "c_crumsort")]
        bench_func!(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            c_crumsort_unstable,
            unstable::c_crumsort,
        );

        #[cfg(feature = "cpp_std_sys")]
        bench_func!(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            cpp_std_sys_unstable,
            unstable::cpp_std_sys,
        );

        #[cfg(feature = "cpp_std_libcxx")]
        bench_func!(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            cpp_std_libcxx_unstable,
            unstable::cpp_std_libcxx,
        );

        // --- Other sorts ---

        #[cfg(feature = "rust_radsort")]
        bench_func!(
            c,
            test_size,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
            rust_radsort_radix,
            sort_comp::other::rust_radsort,
        );
    }
}

// Very large stack value.
#[derive(PartialEq, Eq, Debug, Clone)]
struct OneKiloByte {
    values: [i32; 256],
}

impl OneKiloByte {
    fn new(val: i32) -> Self {
        let mut values = [val; 256];
        values[54] = 6i32.wrapping_mul(val);
        values[100] = 18i32.wrapping_sub(val);
        Self { values }
    }

    fn as_i32(&self) -> i32 {
        self.values[55]
    }
}

impl PartialOrd for OneKiloByte {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_i32().partial_cmp(&other.as_i32())
    }
}

impl Ord for OneKiloByte {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

// 16 byte stack value, with more expensive comparison.
#[derive(PartialEq, Debug, Clone, Copy)]
struct F128 {
    x: f64,
    y: f64,
}

impl F128 {
    fn new(val: i32) -> Self {
        let val_f = (val as f64) + (i32::MAX as f64) + 6.0;

        let x = val_f + 0.1;
        let y = val_f.log(4.1);

        debug_assert!(y < x);

        Self { x, y }
    }
}

// This is kind of hacky, but we know we only have normal comparable floats in there.
impl Eq for F128 {}

impl PartialOrd for F128 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Simulate expensive comparison function.
        let this_div = self.x / self.y;
        let other_div = other.x / other.y;

        this_div.partial_cmp(&other_div)
    }
}

impl Ord for F128 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
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
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 13, 15, 16, 17, 19, 20, 24, 36, 50, 101, 200, 500, 1_000,
        2_048, 10_000, 100_000, 1_000_000,
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
        // Larger type that is not Copy and does heap access.
        bench_patterns(c, test_size, "string", |values| {
            // Strings are compared lexicographically, so we zero extend them to maintain the input
            // order.
            // See: https://godbolt.org/z/M38zTK6nv and https://godbolt.org/z/G18Yb7zoE
            values
                .iter()
                .map(|val| format!("{:010}", val.saturating_abs()))
                .collect()
        });
        // Very large stack value.
        bench_patterns(c, test_size, "1k", |values| {
            values.iter().map(|val| OneKiloByte::new(*val)).collect()
        });
        // 16 byte stack value that is Copy but has a relatively expensive cmp implementation.
        bench_patterns(c, test_size, "f128", |values| {
            values.iter().map(|val| F128::new(*val)).collect()
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
