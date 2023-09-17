use std::env;

use criterion::{criterion_group, criterion_main, Criterion};

#[allow(unused_imports)]
use sort_test_tools::ffi_types::{FFIOneKiloByte, FFIString, F128};

use sort_test_tools::patterns;

#[allow(unused_imports)]
use sort_comp::{stable, unstable};

#[cfg(feature = "cold_benchmarks")]
mod trash_prediction;

mod modules;

use crate::modules::bench_len_type_pattern_combo;

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
    test_len: usize,
    transform_name: &str,
    transform: fn(Vec<i32>) -> Vec<T>,
) {
    if test_len > 100_000 && !(transform_name == "i32" || transform_name == "u64") {
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
        ("saw_ascending", |len| {
            patterns::saw_ascending(len, ((len as f64).log2().round()) as usize)
        }),
        ("saw_descending", |len| {
            patterns::saw_descending(len, ((len as f64).log2().round()) as usize)
        }),
        ("saws_long", |len| {
            patterns::saw_mixed(len, ((len as f64).log2().round()) as usize)
        }),
        ("pipe_organ", patterns::pipe_organ),
        ("random__div3", |len| {
            patterns::random_uniform(len, 0..=(((len as f64 / 3.0).round()) as i32))
        }),
        ("random__div5", |len| {
            patterns::random_uniform(len, 0..=(((len as f64 / 5.0).round()) as i32))
        }),
        ("random__div8", |len| {
            patterns::random_uniform(len, 0..=(((len as f64 / 8.0).round()) as i32))
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

    let args = env::args().collect::<Vec<_>>();
    // No clue how stable that is.
    let filter_arg = &args[args.len() - 2];

    for (pattern_name, pattern_provider) in pattern_providers.iter() {
        if test_len < 3 && *pattern_name != "random" {
            continue;
        }

        bench_len_type_pattern_combo(
            c,
            filter_arg,
            test_len,
            transform_name,
            &transform,
            pattern_name,
            pattern_provider,
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

    for test_len in test_sizes {
        // Basic type often used to test sorting algorithms.
        bench_patterns(c, test_len, "i32", |values| values);

        // Common type for usize on 64-bit machines.
        // Sorting indices is very common.
        bench_patterns(c, test_len, "u64", |values| {
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
        // FFI String
        bench_patterns(c, test_len, "string", |values| {
            values
                .iter()
                .map(|val| FFIString::new(format!("{:010}", val.saturating_abs())))
                .collect()
        });

        // Very large stack value.
        bench_patterns(c, test_len, "1k", |values| {
            values.iter().map(|val| FFIOneKiloByte::new(*val)).collect()
        });

        // 16 byte stack value that is Copy but has a relatively expensive cmp implementation.
        bench_patterns(c, test_len, "f128", |values| {
            values.iter().map(|val| F128::new(*val)).collect()
        });

        #[cfg(feature = "bench_type_rust_string")]
        {
            bench_patterns(c, test_len, "rust_string", |values| {
                // Strings are compared lexicographically, so we zero extend them to maintain the input
                // order.
                // See: https://godbolt.org/z/M38zTK6nv and https://godbolt.org/z/G18Yb7zoE
                values
                    .iter()
                    .map(|val| format!("{:010}", val.saturating_abs()))
                    .collect()
            });
        }

        #[cfg(feature = "bench_type_val_with_mutex")]
        {
            use std::cmp::Ordering;
            use std::sync::Mutex;

            #[derive(Debug)]
            struct ValWithMutex {
                val: u64,
                mutex: Mutex<u64>,
            }

            impl PartialEq for ValWithMutex {
                fn eq(&self, other: &Self) -> bool {
                    self.val == other.val
                }
            }

            impl Eq for ValWithMutex {}

            impl PartialOrd for ValWithMutex {
                fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                    self.val.partial_cmp(&other.val)
                }
            }

            impl Ord for ValWithMutex {
                fn cmp(&self, other: &Self) -> Ordering {
                    self.partial_cmp(other).unwrap()
                }
            }

            bench_patterns(c, test_len, "val_with_mutex", |values| {
                values
                    .iter()
                    .map(|val| -> ValWithMutex {
                        let mut val_u64 = ((*val as i64) + (i32::MAX as i64) + 1) as u64;
                        val_u64 = val_u64.checked_mul(i32::MAX as u64).unwrap();

                        let mut this = ValWithMutex {
                            val: val_u64,
                            mutex: Mutex::new(val_u64),
                        };

                        // To make sure mutex is not optimized away.
                        this.val = *this.mutex.lock().unwrap();

                        this
                    })
                    .collect()
            });
        }

        #[cfg(feature = "bench_type_u8")]
        {
            bench_patterns(c, test_len, "u8", |values| {
                values
                    .iter()
                    .map(|val| -> u8 { (val & u8::MAX as i32) as u8 })
                    .collect()
            });
        }

        #[cfg(feature = "bench_type_u16")]
        {
            bench_patterns(c, test_len, "u16", |values| {
                values
                    .iter()
                    .map(|val| -> u16 { (val & u16::MAX as i32) as u16 })
                    .collect()
            });
        }

        #[cfg(feature = "bench_type_u32")]
        {
            bench_patterns(c, test_len, "u32", |values| {
                values
                    .iter()
                    .map(|val| -> u32 { (val & u32::MAX as i32) as u32 })
                    .collect()
            });
        }

        #[cfg(feature = "bench_type_u128")]
        {
            bench_patterns(c, test_len, "u128", |values| {
                values
                    .iter()
                    .map(|val| -> u128 {
                        // Extends the value into the 128 bit range,
                        // while preserving input order.
                        let x = ((*val as i128) + (i32::MAX as i128) + 1) as u128;
                        x.checked_mul(i64::MAX as u128).unwrap()
                    })
                    .collect()
            });
        }
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
