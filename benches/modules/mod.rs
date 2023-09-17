use std::env;

use criterion::Criterion;

pub mod sort;

#[cfg(feature = "partition_point")]
pub mod partition_point;

#[cfg(feature = "partition")]
pub mod partition;

#[allow(unused)]
pub fn bench_len_type_pattern_combo<T: Ord + std::fmt::Debug>(
    c: &mut Criterion,
    filter_arg: &str,
    test_len: usize,
    transform_name: &str,
    transform: &fn(Vec<i32>) -> Vec<T>,
    pattern_name: &str,
    pattern_provider: &fn(usize) -> Vec<i32>,
) {
    if let Ok(env_val) = env::var("BENCH_OTHER") {
        match env_val.as_str() {
            #[cfg(feature = "partition_point")]
            "partition_point" => {
                partition_point::bench(
                    c,
                    filter_arg,
                    test_len,
                    transform_name,
                    transform,
                    pattern_name,
                    pattern_provider,
                );
            }
            #[cfg(feature = "partition")]
            "partition" => {
                partition::bench(
                    c,
                    filter_arg,
                    test_len,
                    transform_name,
                    transform,
                    pattern_name,
                    pattern_provider,
                );
            }
            _ => panic!(
                "Unknown BENCH_OTHER value: '{}'. Make sure the feature is enabled.",
                env_val
            ),
        }
    } else {
        sort::bench(
            c,
            filter_arg,
            test_len,
            transform_name,
            transform,
            pattern_name,
            pattern_provider,
        );
    }
}

pub mod util;
