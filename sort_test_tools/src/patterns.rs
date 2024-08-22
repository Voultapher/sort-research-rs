use std::env;
use std::str::FromStr;
use std::sync::Mutex;

use rand::prelude::*;

use zipf::ZipfDistribution;

/// Provides a set of patterns useful for testing and benchmarking sorting algorithms.
/// Currently limited to i32 values.

// --- Public ---

pub fn random(len: usize) -> Vec<i32> {
    //     .
    // : . : :
    // :.:::.::

    random_vec(len)
}

pub fn random_uniform<R>(len: usize, range: R) -> Vec<i32>
where
    R: Into<rand::distributions::Uniform<i32>>,
{
    // :.:.:.::
    let mut rng = rand::rngs::StdRng::from(new_seed());

    // Abstracting over ranges in Rust :(
    let dist: rand::distributions::Uniform<i32> = range.into();

    (0..len).map(|_| dist.sample(&mut rng)).collect()
}

pub fn random_zipf(len: usize, exponent: f64) -> Vec<i32> {
    // https://en.wikipedia.org/wiki/Zipf's_law
    let mut rng = rand::rngs::StdRng::from(new_seed());

    // Abstracting over ranges in Rust :(
    let dist = ZipfDistribution::new(len, exponent).unwrap();

    (0..len).map(|_| dist.sample(&mut rng) as i32).collect()
}

pub fn random_sorted(len: usize, sorted_percent: f64) -> Vec<i32> {
    //     .:
    //   .:::. :
    // .::::::.::
    // [----][--]
    //  ^      ^
    //  |      |
    // sorted  |
    //     unsorted

    // Simulate pre-existing sorted slice, where len - sorted_percent are the new unsorted values
    // and part of the overall distribution.
    let mut v = random_vec(len);
    let sorted_len = ((len as f64) * (sorted_percent / 100.0)).round() as usize;

    v[0..sorted_len].sort_unstable();

    v
}

pub fn random_random_size(max_size: usize) -> Vec<i32> {
    //     .
    // : . : :
    // :.:::.::
    // < len > is random from call to call, with max_size as maximum len.

    let random_size = random_uniform(1, 0..=(max_size as i32));
    random(random_size[0] as usize)
}

pub fn all_equal(len: usize) -> Vec<i32> {
    // ......
    // ::::::

    (0..len).map(|_| 66).collect::<Vec<_>>()
}

pub fn ascending(len: usize) -> Vec<i32> {
    //     .:
    //   .:::
    // .:::::

    (0..len as i32).collect::<Vec<_>>()
}

pub fn descending(len: usize) -> Vec<i32> {
    // :.
    // :::.
    // :::::.

    (0..len as i32).rev().collect::<Vec<_>>()
}

pub fn saw_ascending(len: usize, saw_count: usize) -> Vec<i32> {
    //   .:  .:
    // .:::.:::

    if len == 0 {
        return Vec::new();
    }

    let mut vals = random_vec(len);
    let chunks_size = len / saw_count.max(1);

    for chunk in vals.chunks_mut(chunks_size) {
        chunk.sort();
    }

    vals
}

pub fn saw_descending(len: usize, saw_count: usize) -> Vec<i32> {
    // :.  :.
    // :::.:::.

    if len == 0 {
        return Vec::new();
    }

    let mut vals = random_vec(len);
    let chunks_size = len / saw_count.max(1);

    for chunk in vals.chunks_mut(chunks_size) {
        chunk.sort_by_key(|&e| std::cmp::Reverse(e));
    }

    vals
}

pub fn saw_mixed(len: usize, saw_count: usize) -> Vec<i32> {
    // :.  :.    .::.    .:
    // :::.:::..::::::..:::

    if len == 0 {
        return Vec::new();
    }

    let mut vals = random_vec(len);
    let chunks_size = len / saw_count.max(1);
    let saw_directions = random_uniform((len / chunks_size) + 1, 0..=1);

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

pub fn saw_mixed_range(len: usize, range: std::ops::Range<usize>) -> Vec<i32> {
    //     :.
    // :.  :::.    .::.      .:
    // :::.:::::..::::::..:.:::

    // ascending and descending randomly picked, with length in `range`.

    if len == 0 {
        return Vec::new();
    }

    let mut vals = random_vec(len);

    let max_chunks = len / range.start;
    let saw_directions = random_uniform(max_chunks + 1, 0..=1);
    let chunk_sizes = random_uniform(max_chunks + 1, (range.start as i32)..(range.end as i32));

    let mut i = 0;
    let mut l = 0;
    while l < len {
        let chunk_size = chunk_sizes[i] as usize;
        let chunk_end = std::cmp::min(l + chunk_size, len);
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

pub fn pipe_organ(len: usize) -> Vec<i32> {
    //   .:.
    // .:::::.

    let mut vals = random_vec(len);

    let first_half = &mut vals[0..(len / 2)];
    first_half.sort();

    let second_half = &mut vals[(len / 2)..len];
    second_half.sort_by_key(|&e| std::cmp::Reverse(e));

    vals
}

/// Overwrites the default behavior so that each call to a random derived pattern yields new random
/// values.
///
/// By default `patterns::random(4)` will yield the same values per process invocation.
/// For benchmarks it's advised to use call this function.
pub fn use_random_seed_each_time() {
    let (seed_type, _) = get_or_init_seed_type_and_value();
    if seed_type == SeedType::ExternalOverride {
        panic!("Using use_random_seed_each_time conflicts with the external seed override.");
    }

    *SEED_TYPE_AND_VALUE.lock().unwrap() = Some((SeedType::RandomEachTime, 0));
}

pub fn random_init_seed() -> u64 {
    get_or_init_seed_type_and_value().1
}

// --- Private ---

#[derive(Copy, Clone, PartialEq, Eq)]
enum SeedType {
    RandomEachTime,
    RandomOncePerProcess,
    ExternalOverride,
}

static SEED_TYPE_AND_VALUE: Mutex<Option<(SeedType, u64)>> = Mutex::new(None);

fn get_or_init_seed_type_and_value() -> (SeedType, u64) {
    let (seed_type, seed_val) = *SEED_TYPE_AND_VALUE.lock().unwrap().get_or_insert_with(|| {
        if let Some(override_seed) = env::var("OVERRIDE_SEED")
            .ok()
            .map(|seed| u64::from_str(&seed).unwrap())
        {
            (SeedType::ExternalOverride, override_seed)
        } else {
            let per_process_seed = thread_rng().gen();
            (SeedType::RandomOncePerProcess, per_process_seed)
        }
    });

    if seed_type == SeedType::RandomEachTime {
        (SeedType::RandomEachTime, thread_rng().gen())
    } else {
        (seed_type, seed_val)
    }
}

fn new_seed() -> StdRng {
    // Random seed, but prints it for repeatability.
    rand::SeedableRng::seed_from_u64(random_init_seed())
}

fn random_vec(len: usize) -> Vec<i32> {
    let mut rng = rand::rngs::StdRng::from(new_seed());

    (0..len).map(|_| rng.gen::<i32>()).collect()
}
