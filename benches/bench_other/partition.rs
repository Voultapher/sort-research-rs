use std::collections::HashSet;
use std::mem;
use std::ptr;
use std::sync::Mutex;
use std::time;

use criterion::{black_box, Criterion};

use sort_comp::other::partition::{self, Partition};

use crate::bench_other::util::{cpu_max_freq_hz, pin_thread_to_core};

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_unstable_by(|a, b| a.total_cmp(b));
    let median_item = ((values.len() as f64 + 1.0) / 2.0).round();
    values[std::cmp::min(median_item as usize, values.len() - 1)]
}

fn bench_partition_impl<T: Ord + std::fmt::Debug, P: Partition>(
    filter_arg: &str,
    test_size: usize,
    transform_name: &str,
    transform: &fn(Vec<i32>) -> Vec<T>,
    pattern_name: &str,
    pattern_provider: &fn(usize) -> Vec<i32>,
    _partition_impl: P,
) {
    // Pin the benchmark to the same core to improve repeatability. Doing it this way allows
    // criterion to do other stuff with other threads, which greatly impacts overall benchmark
    // throughput.
    pin_thread_to_core();

    let bench_name = format!(
        "{}-{}-{}-{}-",
        P::name(),
        transform_name,
        pattern_name,
        test_size
    );

    if !bench_name.contains(filter_arg) {
        return;
    }

    if test_size > 1_000_000 {
        eprintln!("Test size too large: {test_size}");
        return;
    }

    let input_bytes = mem::size_of::<T>() * test_size;
    let mut batch_size = if input_bytes > 100_000_000 {
        5
    } else if input_bytes > 1_000_000 {
        100
    } else {
        1000
    };

    // Partition time should be roughly linear with input size.
    let test_runs = std::cmp::max(100_000_000 / test_size, 200);
    let batched_runs = std::cmp::max(test_runs / batch_size, 1);

    if test_runs < batch_size {
        // Eg. 500 < 1000, avoid wasting time and memory.
        batch_size = test_runs;
    }

    let mut time_measurements = Vec::with_capacity(batched_runs);
    let mut side_effect = 0;

    for i in 0..(batched_runs + 1) {
        let mut test_inputs = (0..batch_size)
            .map(|_| transform(pattern_provider(test_size)))
            .collect::<Vec<_>>();

        let start = time::Instant::now();

        for test_input in &mut test_inputs {
            // Uncomment for random pivot, potentially pretty uneven.
            let pivot_pos = choose_pivot(test_input, &mut |a, b| a.lt(b));
            test_input.swap(0, pivot_pos);

            let pivot = unsafe { mem::ManuallyDrop::new(ptr::read(&test_input[0])) };
            let swap_idx = black_box(P::partition(
                black_box(&mut test_input[1..]),
                black_box(&pivot),
            ));

            // side-effect
            if swap_idx < test_input.len() {
                test_input.swap(0, swap_idx);
            }
            unsafe {
                if test_input.get_unchecked(3) > test_input.get_unchecked(test_size - 1) {
                    side_effect += 1;
                }
            }
        }

        let end = time::Instant::now();
        if i != 0 {
            // Ignore first run.
            time_measurements.push(end - start);
        }
    }

    let median_elem_per_ns = median(
        time_measurements
            .into_iter()
            .map(|time_diff| test_size as f64 / (time_diff.as_nanos() as f64 / batch_size as f64))
            .collect(),
    );

    if side_effect == test_runs {
        println!("side effect triggered");
    }

    if let Some(max_freq_hz) = cpu_max_freq_hz() {
        let median_elem_per_cycle = median_elem_per_ns / (max_freq_hz / 1_000_000_000.0);
        println!("{bench_name: <50} {median_elem_per_cycle:.2} elem/cycle");
    } else {
        println!("{bench_name: <50} {median_elem_per_ns:.2} elem/ns");
    }
}

/// Selects a pivot from left, right.
///
/// Idea taken from glidesort by Orson Peters.
///
/// This chooses a pivot by sampling an adaptive amount of points, mimicking the median quality of
/// median of square root.
fn choose_pivot<T, F>(v: &[T], is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    let len_div_2 = len / 2;
    let arr_ptr = v.as_ptr();

    let median_guess_ptr = if len < PSEUDO_MEDIAN_REC_THRESHOLD {
        // For small sizes it's crucial to pick a good median, just doing median3 is not great.
        let start = len_div_2 - 3;
        median7_approx(&v[start..(start + 7)], is_less)
    } else {
        // SAFETY: TODO
        unsafe {
            let len_div_8 = len / 8;
            let a = arr_ptr;
            let b = arr_ptr.add(len_div_8 * 4);
            let c = arr_ptr.add(len_div_8 * 7);

            median3_rec(a, b, c, len_div_8, is_less)
        }
    };

    // SAFETY: median_guess_ptr is part of v if median7_approx and median3_rec work as expected.
    unsafe { median_guess_ptr.offset_from(arr_ptr) as usize }
}

// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
// performance impact.
#[inline(never)]
fn median7_approx<T, F>(v: &[T], is_less: &mut F) -> *const T
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 7.
    assert!(v.len() == 7);

    let arr_ptr = v.as_ptr();

    // We checked the len.
    unsafe {
        let lower_median3 = median3(arr_ptr.add(0), arr_ptr.add(1), arr_ptr.add(2), is_less);
        let upper_median3 = median3(arr_ptr.add(4), arr_ptr.add(5), arr_ptr.add(6), is_less);

        let median_approx_ptr = median3(lower_median3, arr_ptr.add(3), upper_median3, is_less);
        median_approx_ptr
    }
}

const PSEUDO_MEDIAN_REC_THRESHOLD: usize = 64;

/// Calculates an approximate median of 3 elements from sections a, b, c, or recursively from an
/// approximation of each, if they're large enough. By dividing the size of each section by 8 when
/// recursing we have logarithmic recursion depth and overall sample from
/// f(n) = 3*f(n/8) -> f(n) = O(n^(log(3)/log(8))) ~= O(n^0.528) elements.
///
/// SAFETY: a, b, c must point to the start of initialized regions of memory of
/// at least n elements.
#[inline(never)]
unsafe fn median3_rec<T, F>(
    mut a: *const T,
    mut b: *const T,
    mut c: *const T,
    n: usize,
    is_less: &mut F,
) -> *const T
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: TODO
    unsafe {
        if n * 8 >= PSEUDO_MEDIAN_REC_THRESHOLD {
            let n8 = n / 8;
            a = median3_rec(a, a.add(n8 * 4), a.add(n8 * 7), n8, is_less);
            b = median3_rec(b, b.add(n8 * 4), b.add(n8 * 7), n8, is_less);
            c = median3_rec(c, c.add(n8 * 4), c.add(n8 * 7), n8, is_less);
        }
        median3(a, b, c, is_less)
    }
}

/// Calculates the median of 3 elements.
///
/// SAFETY: a, b, c must be valid initialized elements.
unsafe fn median3<T, F>(a: *const T, b: *const T, c: *const T, is_less: &mut F) -> *const T
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: TODO
    //
    // Compiler tends to make this branchless when sensible, and avoids the
    // third comparison when not.
    unsafe {
        let x = is_less(&*a, &*b);
        let y = is_less(&*a, &*c);
        if x == y {
            // If x=y=0 then b, c <= a. In this case we want to return max(b, c).
            // If x=y=1 then a < b, c. In this case we want to return min(b, c).
            // By toggling the outcome of b < c using XOR x we get this behavior.
            let z = is_less(&*b, &*c);

            if z ^ x {
                c
            } else {
                b
            }
        } else {
            // Either c <= a < b or b <= a < c, thus a is our median.
            a
        }
    }
}

pub fn bench<T: Ord + std::fmt::Debug>(
    _c: &mut Criterion,
    filter_arg: &str,
    test_size: usize,
    transform_name: &str,
    transform: &fn(Vec<i32>) -> Vec<T>,
    pattern_name: &str,
    pattern_provider: &fn(usize) -> Vec<i32>,
) {
    // We are not really interested in very small input. These are handled by some other logic.
    if test_size < 30 {
        return;
    }

    static SEEN_SIZES: Mutex<Option<HashSet<usize>>> = Mutex::new(None);

    let mut seen_sizes = SEEN_SIZES.lock().unwrap();
    if seen_sizes.is_none() {
        *seen_sizes = Some(HashSet::new());
    }

    let seen_before = !seen_sizes.as_mut().unwrap().insert(test_size);
    if !seen_before {
        println!(""); // For readability to split multiple blocks.
    }

    // TODO use proper criterion benchmarking.

    // bench_partition_impl(
    //     filter_arg,
    //     test_size,
    //     transform_name,
    //     transform,
    //     pattern_name,
    //     pattern_provider,
    //     partition::sum_is_less::PartitionImpl,
    // );

    // bench_partition_impl(
    //     filter_arg,
    //     test_size,
    //     transform_name,
    //     transform,
    //     pattern_name,
    //     pattern_provider,
    //     partition::sum_lookup::PartitionImpl,
    // );

    // bench_partition_impl(
    //     filter_arg,
    //     test_size,
    //     transform_name,
    //     transform,
    //     pattern_name,
    //     pattern_provider,
    //     partition::simple_scan_branchy::PartitionImpl,
    // );

    // bench_partition_impl(
    //     filter_arg,
    //     test_size,
    //     transform_name,
    //     transform,
    //     pattern_name,
    //     pattern_provider,
    //     partition::simple_scan_branchless::PartitionImpl,
    // );

    // bench_partition_impl(
    //     filter_arg,
    //     test_size,
    //     transform_name,
    //     transform,
    //     pattern_name,
    //     pattern_provider,
    //     partition::scan_branchless_2unroll::PartitionImpl,
    // );

    bench_partition_impl(
        filter_arg,
        test_size,
        transform_name,
        transform,
        pattern_name,
        pattern_provider,
        partition::block_quicksort::PartitionImpl,
    );

    // bench_partition_impl(
    //     filter_arg,
    //     test_size,
    //     transform_name,
    //     transform,
    //     pattern_name,
    //     pattern_provider,
    //     partition::fulcrum_partition_simple::PartitionImpl,
    // );

    bench_partition_impl(
        filter_arg,
        test_size,
        transform_name,
        transform,
        pattern_name,
        pattern_provider,
        partition::fulcrum_partition_scandum::PartitionImpl,
    );

    // bench_partition_impl(
    //     filter_arg,
    //     test_size,
    //     transform_name,
    //     transform,
    //     pattern_name,
    //     pattern_provider,
    //     partition::crumsort::PartitionImpl,
    // );

    // bench_partition_impl(
    //     filter_arg,
    //     test_size,
    //     transform_name,
    //     transform,
    //     pattern_name,
    //     pattern_provider,
    //     partition::new_block_quicksort::PartitionImpl,
    // );

    // bench_partition_impl(
    //     filter_arg,
    //     test_size,
    //     transform_name,
    //     transform,
    //     pattern_name,
    //     pattern_provider,
    //     partition::small_fast::PartitionImpl,
    // );

    // bench_partition_impl(
    //     filter_arg,
    //     test_size,
    //     transform_name,
    //     transform,
    //     pattern_name,
    //     pattern_provider,
    //     partition::ilp_partition::PartitionImpl,
    // );

    // bench_partition_impl(
    //     filter_arg,
    //     test_size,
    //     transform_name,
    //     transform,
    //     pattern_name,
    //     pattern_provider,
    //     partition::avx2::PartitionImpl,
    // );

    // bench_partition_impl(
    //     filter_arg,
    //     test_size,
    //     transform_name,
    //     transform,
    //     pattern_name,
    //     pattern_provider,
    //     partition::scan_branchless_2way::PartitionImpl,
    // );

    // bench_partition_impl(
    //     filter_arg,
    //     test_size,
    //     transform_name,
    //     transform,
    //     pattern_name,
    //     pattern_provider,
    //     partition::scan_branchless_4way::PartitionImpl,
    // );

    // bench_partition_impl(
    //     filter_arg,
    //     test_size,
    //     transform_name,
    //     transform,
    //     pattern_name,
    //     pattern_provider,
    //     partition::rotate_branchless_2way::PartitionImpl,
    // );

    // bench_partition_impl(
    //     filter_arg,
    //     test_size,
    //     transform_name,
    //     transform,
    //     pattern_name,
    //     pattern_provider,
    //     partition::fulcrum_partition_revised::PartitionImpl,
    // );

    // bench_partition_impl(
    //     filter_arg,
    //     test_size,
    //     transform_name,
    //     transform,
    //     pattern_name,
    //     pattern_provider,
    //     partition::luna_partition::PartitionImpl,
    // );

    bench_partition_impl(
        filter_arg,
        test_size,
        transform_name,
        transform,
        pattern_name,
        pattern_provider,
        partition::butterfly_partition::PartitionImpl,
    );
}
