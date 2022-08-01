use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

use sort_comp::patterns;

fn bench_impl<T>(c: &mut Criterion, name: &str, sort_func: impl Fn(&mut [i32]) -> T) {
    // let test_sizes = [1, 2, 5, 8, 15, 200];

    let test_sizes = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 16, 17, 20, 24, 28, 30, 35, 36, 50, 100, 200,
        500, 1_000, 2_048, 10_000, 100_000,
    ];

    let pattern_providers = [
        patterns::random,
        |test_size| patterns::random_uniform(test_size, 0..(test_size / 10) as i32),
        patterns::all_equal,
        patterns::ascending,
        patterns::descending,
        |test_size| patterns::ascending_saw(test_size, test_size / 5),
        |test_size| patterns::ascending_saw(test_size, test_size / 20),
        |test_size| patterns::descending_saw(test_size, test_size / 5),
        |test_size| patterns::descending_saw(test_size, test_size / 20),
        patterns::pipe_organ,
    ];

    let pattern_names = [
        "random",
        "random_uniform",
        "all_equal",
        "ascending",
        "descending",
        "ascending_saw_5",
        "ascending_saw_20",
        "descending_saw_5",
        "descending_saw_20",
        "pipe_organ",
    ];

    for test_size in test_sizes {
        for (pattern_provider, pattern_name) in pattern_providers.iter().zip(pattern_names.iter()) {
            if test_size < 3 && *pattern_name != "random" {
                continue;
            }

            let batch_size = if test_size > 30 {
                BatchSize::LargeInput
            } else {
                BatchSize::SmallInput
            };

            c.bench_function(&format!("{name}-{pattern_name}-{test_size}"), |b| {
                b.iter_batched(
                    || pattern_provider(test_size),
                    |mut test_data| sort_func(black_box(test_data.as_mut_slice())),
                    batch_size,
                )
            });
        }
    }

    {
        // This benchmark aims to avoid having perfect branch prediction in smaller sub-size
        // selection.
        let random_size_range = 0..12;
        c.bench_function(
            &format!("{name}-random-sizes-{:?}", random_size_range),
            |b| {
                b.iter_batched(
                    || {
                        let pick = patterns::random_uniform(1, random_size_range.clone());
                        patterns::random(pick[0] as usize)
                    },
                    |mut test_data| sort_func(black_box(test_data.as_mut_slice())),
                    BatchSize::SmallInput,
                )
            },
        );
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    bench_impl(c, "flux", |arr| {
        sort_comp::fluxsort::sort_by(arr, |a, b| a.cmp(b))
    });
    bench_impl(c, "std_stable", |arr| arr.sort());
    bench_impl(c, "std_unstable", |arr| arr.sort_unstable());
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
