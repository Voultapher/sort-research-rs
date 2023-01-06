use std::collections::HashSet;
use std::env;
use std::mem;
use std::ptr;
use std::str::FromStr;
use std::sync::Mutex;
use std::time;

use once_cell::sync::OnceCell;

use criterion::black_box;

use sort_comp::other::partition::{self, Partition};

fn cpu_max_freq_hz() -> Option<f64> {
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

pub fn bench_custom<T: Ord + std::fmt::Debug>(
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

    static SEEN_BENCHMARKS: Mutex<Option<HashSet<String>>> = Mutex::new(None);

    let mut seen_benchmarks = SEEN_BENCHMARKS.lock().unwrap();

    if seen_benchmarks.is_none() {
        *seen_benchmarks = Some(HashSet::new());
    }

    // Kind of hacky way to deduplicate
    let seen_before = !seen_benchmarks
        .as_mut()
        .unwrap()
        .insert(format!("{}-{}-{}", transform_name, pattern_name, test_size));

    if seen_before {
        return;
    }

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

    // ---

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
    //     partition::crumsort::PartitionImpl,
    // );

    bench_partition_impl(
        filter_arg,
        test_size,
        transform_name,
        transform,
        pattern_name,
        pattern_provider,
        partition::new_block_quicksort::PartitionImpl,
    );

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
}
