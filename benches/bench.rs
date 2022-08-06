use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

use sort_comp::patterns;

fn bench_patterns<T: Ord>(
    c: &mut Criterion,
    test_size: usize,
    transform_name: &str,
    transform: fn(Vec<i32>) -> Vec<T>,
) {
    let pattern_providers = [
        patterns::random,
        |size| patterns::random_uniform(size, 0..(size / 10) as i32),
        patterns::random_random_size,
        patterns::all_equal,
        patterns::ascending,
        patterns::descending,
        |size| patterns::ascending_saw(size, size / 5),
        |size| patterns::ascending_saw(size, size / 20),
        |size| patterns::descending_saw(size, size / 5),
        |size| patterns::descending_saw(size, size / 20),
        patterns::pipe_organ,
    ];

    let pattern_names = [
        "random",
        "random_uniform",
        "random_random_size",
        "all_equal",
        "ascending",
        "descending",
        "ascending_saw_5",
        "ascending_saw_20",
        "descending_saw_5",
        "descending_saw_20",
        "pipe_organ",
        "variable_size_0_to",
    ];

    for (pattern_provider, pattern_name) in pattern_providers.iter().zip(pattern_names.iter()) {
        if test_size < 3 && *pattern_name != "random" {
            continue;
        }

        let batch_size = if test_size > 30 {
            BatchSize::LargeInput
        } else {
            BatchSize::SmallInput
        };

        use sort_comp::new_stable_sort;
        c.bench_function(
            &format!("new_stable-{transform_name}-{pattern_name}-{test_size}"),
            |b| {
                b.iter_batched(
                    || transform(pattern_provider(test_size)),
                    |mut test_data| new_stable_sort::sort(test_data.as_mut_slice()),
                    batch_size,
                )
            },
        );

        use sort_comp::stdlib_stable;
        c.bench_function(
            &format!("std_stable-{transform_name}-{pattern_name}-{test_size}"),
            |b| {
                b.iter_batched(
                    || transform(pattern_provider(test_size)),
                    |mut test_data| stdlib_stable::sort(test_data.as_mut_slice()),
                    batch_size,
                )
            },
        );

        // use sort_comp::stdlib_unstable;
        // c.bench_function(
        //     &format!("std_unstable-{transform_name}-{pattern_name}-{test_size}"),
        //     |b| {
        //         b.iter_batched(
        //             || transform(pattern_provider(test_size)),
        //             |mut test_data| stdlib_unstable::sort_unstable(test_data.as_mut_slice()),
        //             batch_size,
        //         )
        //     },
        // );
    }
}

// Very large stack value.
#[derive(PartialEq, Eq)]
struct OneKiloByte {
    values: [i32; 256],
}

impl OneKiloByte {
    fn new(val: i32) -> Self {
        Self { values: [val; 256] }
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

// Very large stack value.
#[derive(PartialEq)]
struct F64 {
    value: f64,
}

impl F64 {
    fn new(val: i32) -> Self {
        Self {
            value: val as f64 + 0.1,
        }
    }
}

// This is kind of hacky, but we know we only have normal comparable floats in there.
impl Eq for F64 {}

impl PartialOrd for F64 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for F64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let test_sizes = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 16, 17, 20, 24, 28, 30, 35, 36, 50, 101, 200,
        500, 1_000, 2_048, 10_000, 100_000, 1_000_000,
    ];

    for test_size in test_sizes {
        bench_patterns(c, test_size, "i32", |values| values);
        bench_patterns(c, test_size, "string", |values| {
            values.iter().map(|val| val.to_string()).collect()
        });
        bench_patterns(c, test_size, "1k", |values| {
            values.iter().map(|val| OneKiloByte::new(*val)).collect()
        });
        bench_patterns(c, test_size, "box_str", |values| {
            values
                .iter()
                .map(|val| val.to_string().into_boxed_str())
                .collect()
        });
        bench_patterns(c, test_size, "f64", |values| {
            values.iter().map(|val| F64::new(*val)).collect()
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
