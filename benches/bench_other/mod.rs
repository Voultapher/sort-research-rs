use criterion::Criterion;

#[cfg(feature = "partition_point")]
pub mod partition_point;

#[cfg(feature = "partition")]
pub mod partition;

#[allow(unused)]
pub fn bench_other<T: Ord + std::fmt::Debug>(
    c: &mut Criterion,
    filter_arg: &str,
    test_size: usize,
    transform_name: &str,
    transform: &fn(Vec<i32>) -> Vec<T>,
    pattern_name: &str,
    pattern_provider: &fn(usize) -> Vec<i32>,
) {
    #[cfg(feature = "partition_point")]
    partition_point::bench(
        c,
        filter_arg,
        test_size,
        transform_name,
        transform,
        pattern_name,
        pattern_provider,
    );

    #[cfg(feature = "partition")]
    partition::bench(
        c,
        filter_arg,
        test_size,
        transform_name,
        transform,
        pattern_name,
        pattern_provider,
    );
}

pub mod util;
