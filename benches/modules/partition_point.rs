use std::hint::black_box;

use criterion::Criterion;

use rand::prelude::*;

use sort_comp::other::partition_point::{self, PartitionPoint};

use crate::modules::util::bench_fn;

#[inline(never)]
fn bench_impl<T: Ord + std::fmt::Debug, P: PartitionPoint>(
    c: &mut Criterion,
    test_len: usize,
    transform_name: &str,
    transform: &fn(Vec<i32>) -> Vec<T>,
    pattern_name: &str,
    pattern_provider: &fn(usize) -> Vec<i32>,
    _partition_point_impl: P,
) {
    let bench_name = P::name();

    let p_pattern_provider = |len: usize| -> Vec<i32> {
        // Limit the val to somewhere in the range of the ascending pattern.
        // Using mod is skewed randomness, I think it should be fine in this case.
        let rand_val = (rand::thread_rng().gen::<u32>() % len as u32) as i32;

        // Inject the last value as the random value we will look for.
        let mut v = pattern_provider(len);
        v.push(rand_val);

        v
    };

    let p_test_fn = |v: &mut [T]| {
        let end = v.len() - 1;
        let rand_val = &v[end];

        black_box(P::partition_point(&v[..end], rand_val));
    };

    bench_fn(
        c,
        test_len,
        transform_name,
        transform,
        pattern_name,
        p_pattern_provider,
        &bench_name,
        p_test_fn,
    )
}

pub fn bench<T: Ord + std::fmt::Debug>(
    c: &mut Criterion,
    _filter_arg: &str,
    test_len: usize,
    transform_name: &str,
    transform: &fn(Vec<i32>) -> Vec<T>,
    pattern_name: &str,
    pattern_provider: &fn(usize) -> Vec<i32>,
) {
    if pattern_name != "ascending" {
        // We need sorted inputs.
        return;
    }

    bench_impl(
        c,
        test_len,
        transform_name,
        &transform,
        pattern_name,
        pattern_provider,
        partition_point::branchless_clean::PartitionPointImpl,
    );

    bench_impl(
        c,
        test_len,
        transform_name,
        &transform,
        pattern_name,
        pattern_provider,
        partition_point::std::PartitionPointImpl,
    );

    bench_impl(
        c,
        test_len,
        transform_name,
        &transform,
        pattern_name,
        pattern_provider,
        partition_point::branchless_bitwise::PartitionPointImpl,
    );
}
