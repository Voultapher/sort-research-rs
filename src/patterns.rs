use rand::prelude::*;

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

pub fn random_uniform(size: usize, mut range: std::ops::Range<i32>) -> Vec<i32> {
    // :.:.:.::
    let mut rng = rand::rngs::StdRng::from(new_seed());

    if range.end >= range.start {
        range.end += 1;
    }

    let dist = rand::distributions::Uniform::from(range);

    (0..size).map(|_| dist.sample(&mut rng)).collect()
}

pub fn random_random_size(max_size: usize) -> Vec<i32> {
    //     .
    // : . : :
    // :.:::.::
    // < size > is random from call to call, with max_size as maximum size.

    let random_size = random_uniform(1, 0..(max_size as i32));
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

static SEED: OnceCell<u64> = OnceCell::new();

pub fn random_init_seed() -> u64 {
    *SEED.get_or_init(|| -> u64 { thread_rng().gen() })
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
