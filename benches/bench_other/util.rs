use std::env;
use std::str::FromStr;

use regex::Regex;

use criterion::{black_box, BatchSize, Criterion};

use once_cell::sync::OnceCell;

pub fn pin_thread_to_core() {
    use std::cell::Cell;
    let pin_core_id: usize = 2;

    thread_local! {static AFFINITY_ALREADY_SET: Cell<bool> = Cell::new(false); }

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

#[allow(unused)]
pub fn cpu_max_freq_hz() -> Option<f64> {
    static MAX_FREQUENCY: OnceCell<Option<f64>> = OnceCell::new();

    MAX_FREQUENCY
        .get_or_init(|| {
            // I tried using heim-cpu but that introduced too many dependencies.
            if let Ok(val) = env::var("CPU_MAX_FREQ_GHZ") {
                Some(f64::from_str(&val).unwrap() * 1_000_000_000.0)
            } else {
                eprintln!("Unable to determine max CPU frequency, please provide it via env var CPU_MAX_FREQ_GHZ");
                None
            }
        })
        .clone()
}

#[inline(never)]
pub fn bench_fn<T: Ord + std::fmt::Debug>(
    c: &mut Criterion,
    test_size: usize,
    transform_name: &str,
    transform: &fn(Vec<i32>) -> Vec<T>,
    pattern_name: &str,
    pattern_provider: impl Fn(usize) -> Vec<i32>,
    mut bench_name: &str,
    test_fn: impl Fn(&mut [T]),
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
        if bench_name == match_name {
            bench_name = &name[(split_pos + 1)..];
        }
    }

    let bench_name_hot = format!("{bench_name}-hot-{transform_name}-{pattern_name}-{test_size}");
    if is_bench_name_ok(&bench_name_hot) {
        c.bench_function(&bench_name_hot, |b| {
            b.iter_batched_ref(
                || transform(pattern_provider(test_size)),
                |test_data| {
                    test_fn(black_box(test_data.as_mut_slice()));
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
                b.iter_batched_ref(
                    || {
                        let mut test_ints = pattern_provider(test_size);

                        if test_ints.len() == 0 {
                            return vec![];
                        }

                        // Try as best as possible to trash all prediction state in the CPU, to
                        // simulate calling the benchmark function as part of a larger program.
                        // Caveat, memory caches. We don't want to benchmark how expensive it is to
                        // load something from main memory.
                        let first_val = black_box(crate::trash_prediction::trash_prediction_state(
                            black_box(test_ints[0]),
                        ));

                        // Limit the optimizer in getting rid of trash_prediction_state,
                        // by tying its output to the test input.
                        test_ints[0] = first_val;

                        transform(test_ints)
                    },
                    |test_data| {
                        test_fn(black_box(test_data.as_mut_slice()));
                        black_box(test_data); // side-effect
                    },
                    BatchSize::PerIteration,
                )
            });
        }
    }
}
