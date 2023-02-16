use std::sync::atomic::{AtomicBool, Ordering};

use rand::prelude::*;

use zipf::ZipfDistribution;

use once_cell::sync::OnceCell;

/// Provides a set of patterns useful for testing and benchmarking sorting algorithms.
/// Currently limited to i32 values.

// --- Public ---

pub fn random(size: usize) -> Vec<i32> {
    //     .
    // : . : :
    // :.:::.::

    random_vec(size)
}

pub fn random_uniform<R>(size: usize, range: R) -> Vec<i32>
where
    R: Into<rand::distributions::Uniform<i32>>,
{
    // :.:.:.::
    let mut rng = rand::rngs::StdRng::from(new_seed());

    // Abstracting over ranges in Rust :(
    let dist: rand::distributions::Uniform<i32> = range.into();

    (0..size).map(|_| dist.sample(&mut rng)).collect()
}

pub fn random_zipf(size: usize, exponent: f64) -> Vec<i32> {
    // https://en.wikipedia.org/wiki/Zipf's_law
    let mut rng = rand::rngs::StdRng::from(new_seed());

    // Abstracting over ranges in Rust :(
    let dist = ZipfDistribution::new(size, exponent).unwrap();

    (0..size).map(|_| dist.sample(&mut rng) as i32).collect()
}

pub fn random_random_size(max_size: usize) -> Vec<i32> {
    //     .
    // : . : :
    // :.:::.::
    // < size > is random from call to call, with max_size as maximum size.

    let random_size = random_uniform(1, 0..=(max_size as i32));
    random(random_size[0] as usize)
}

pub fn all_equal(size: usize) -> Vec<i32> {
    // ......
    // ::::::

    (0..size).map(|_| 66).collect::<Vec<_>>()
}

pub fn ascending(size: usize) -> Vec<i32> {
    //     .:
    //   .:::
    // .:::::

    (0..size as i32).collect::<Vec<_>>()
}

pub fn descending(size: usize) -> Vec<i32> {
    // :.
    // :::.
    // :::::.

    (0..size as i32).rev().collect::<Vec<_>>()
}

pub fn ascending_saw(size: usize, saw_count: usize) -> Vec<i32> {
    //   .:  .:
    // .:::.:::

    if size == 0 {
        return Vec::new();
    }

    let mut vals = random_vec(size);
    let chunks_size = size / saw_count.max(1);

    for chunk in vals.chunks_mut(chunks_size) {
        chunk.sort();
    }

    vals
}

pub fn descending_saw(size: usize, saw_count: usize) -> Vec<i32> {
    // :.  :.
    // :::.:::.

    if size == 0 {
        return Vec::new();
    }

    let mut vals = random_vec(size);
    let chunks_size = size / saw_count.max(1);

    for chunk in vals.chunks_mut(chunks_size) {
        chunk.sort_by_key(|&e| std::cmp::Reverse(e));
    }

    vals
}

pub fn saw_mixed(size: usize, saw_count: usize) -> Vec<i32> {
    // :.  :.    .::.    .:
    // :::.:::..::::::..:::

    if size == 0 {
        return Vec::new();
    }

    let mut vals = random_vec(size);
    let chunks_size = size / saw_count.max(1);
    let saw_directions = random_uniform((size / chunks_size) + 1, 0..=1);

    for (i, chunk) in vals.chunks_mut(chunks_size).enumerate() {
        if saw_directions[i] == 0 {
            chunk.sort();
        } else if saw_directions[i] == 1 {
            chunk.sort_by_key(|&e| std::cmp::Reverse(e));
        } else {
            unreachable!();
        }
    }

    vals
}

pub fn saw_mixed_range(size: usize, range: std::ops::Range<usize>) -> Vec<i32> {
    //     :.
    // :.  :::.    .::.      .:
    // :::.:::::..::::::..:.:::

    // ascending and descending randomly picked, with length in `range`.

    if size == 0 {
        return Vec::new();
    }

    let mut vals = random_vec(size);

    let max_chunks = size / range.start;
    let saw_directions = random_uniform(max_chunks + 1, 0..=1);
    let chunk_sizes = random_uniform(max_chunks + 1, (range.start as i32)..(range.end as i32));

    let mut i = 0;
    let mut l = 0;
    while l < size {
        let chunk_size = chunk_sizes[i] as usize;
        let chunk_end = std::cmp::min(l + chunk_size, size);
        let chunk = &mut vals[l..chunk_end];

        if saw_directions[i] == 0 {
            chunk.sort();
        } else if saw_directions[i] == 1 {
            chunk.sort_by_key(|&e| std::cmp::Reverse(e));
        } else {
            unreachable!();
        }

        i += 1;
        l += chunk_size;
    }

    vals
}

pub fn pipe_organ(size: usize) -> Vec<i32> {
    //   .:.
    // .:::::.

    let mut vals = random_vec(size);

    let first_half = &mut vals[0..(size / 2)];
    first_half.sort();

    let second_half = &mut vals[(size / 2)..size];
    second_half.sort_by_key(|&e| std::cmp::Reverse(e));

    vals
}

static USE_FIXED_SEED: AtomicBool = AtomicBool::new(true);

pub fn disable_fixed_seed() {
    USE_FIXED_SEED.store(false, Ordering::Release);
}

pub fn random_init_seed() -> u64 {
    // return 360013155987181959;
    if USE_FIXED_SEED.load(Ordering::Acquire) {
        static SEED: OnceCell<u64> = OnceCell::new();
        *SEED.get_or_init(|| -> u64 { thread_rng().gen() })
    } else {
        thread_rng().gen()
    }
}

// --- Private ---

fn new_seed() -> StdRng {
    // Random seed, but prints it for repeatability.
    rand::SeedableRng::seed_from_u64(random_init_seed())
}

fn random_vec(size: usize) -> Vec<i32> {
    let mut rng = rand::rngs::StdRng::from(new_seed());

    (0..size).map(|_| rng.gen::<i32>()).collect()
}
