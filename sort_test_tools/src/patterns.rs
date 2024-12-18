use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::env;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

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
    R: Into<rand::distributions::Uniform<i32>> + Hash,
{
    // :.:.:.::

    static CACHE: KeyedVecCache = KeyedVecCache::new();

    CACHE.copy_cached_or_gen(len, range, |len, seed, range| {
        let mut rng: StdRng = rand::SeedableRng::seed_from_u64(seed);

        // Abstracting over ranges in Rust :(
        let dist: rand::distributions::Uniform<i32> = range.into();
        (0..len).map(|_| dist.sample(&mut rng)).collect()
    })
}

pub fn random_zipf(len: usize, exponent: f64) -> Vec<i32> {
    // https://en.wikipedia.org/wiki/Zipf's_law

    static CACHE: KeyedVecCache = KeyedVecCache::new();

    CACHE.copy_cached_or_gen(len, exponent.to_bits(), |len, seed, exponent_bits| {
        let mut rng: StdRng = rand::SeedableRng::seed_from_u64(seed);

        // Abstracting over ranges in Rust :(
        let dist = ZipfDistribution::new(len, f64::from_bits(exponent_bits)).unwrap();
        (0..len).map(|_| dist.sample(&mut rng) as i32).collect()
    })
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

    static CACHE: KeyedVecCache = KeyedVecCache::new();

    let spb = sorted_percent.to_bits();
    CACHE.copy_cached_or_gen(len, spb, |len, _seed, spb| {
        // Simulate pre-existing sorted slice, where len - sorted_percent are the new unsorted values
        // and part of the overall distribution.
        let sorted_percent = f64::from_bits(spb);
        let mut v = random_vec(len);
        let sorted_len = ((len as f64) * (sorted_percent / 100.0)).round() as usize;

        v[0..sorted_len].sort_unstable();

        v
    })
}

pub fn random_merge(len: usize, first_run_percent: f64) -> Vec<i32> {
    //     .:
    //   .:::.  :
    // .::::::.::
    // [----][--]
    //  ^      ^
    //  |      |
    // sorted  |
    //     sorted

    static CACHE: KeyedVecCache = KeyedVecCache::new();

    let frp = first_run_percent.to_bits();
    CACHE.copy_cached_or_gen(len, frp, |len, _seed, frp| {
        // Simulate pre-existing sorted slice, where len - first_run_percent are the new unsorted
        // values and part of the overall distribution.
        let first_run_percent = f64::from_bits(frp);
        let mut v = random_vec(len);
        let first_run_len = ((len as f64) * (first_run_percent / 100.0)).round() as usize;

        v[0..first_run_len].sort_unstable();
        v[first_run_len..].sort_unstable();

        v
    })
}

pub fn random_random_size(max_len: usize) -> Vec<i32> {
    //     .
    // : . : :
    // :.:::.::
    // < len > is random from call to call, with max_len as maximum len.

    let random_size = random_uniform(1, 0..=(max_len as i32));
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

    static CACHE: KeyedVecCache = KeyedVecCache::new();

    CACHE.copy_cached_or_gen(len, saw_count, |len, _seed, saw_count| {
        let mut vals = random_vec(len);
        let chunks_size = len / saw_count.max(1);

        for chunk in vals.chunks_mut(chunks_size) {
            chunk.sort_unstable();
        }

        vals
    })
}

pub fn saw_descending(len: usize, saw_count: usize) -> Vec<i32> {
    // :.  :.
    // :::.:::.

    if len == 0 {
        return Vec::new();
    }

    static CACHE: KeyedVecCache = KeyedVecCache::new();

    CACHE.copy_cached_or_gen(len, saw_count, |len, _seed, saw_count| {
        let mut vals = random_vec(len);
        let chunks_size = len / saw_count.max(1);

        for chunk in vals.chunks_mut(chunks_size) {
            chunk.sort_unstable_by_key(|&e| std::cmp::Reverse(e));
        }

        vals
    })
}

pub fn saw_mixed(len: usize, saw_count: usize) -> Vec<i32> {
    // :.  :.    .::.    .:
    // :::.:::..::::::..:::

    if len == 0 {
        return Vec::new();
    }

    static CACHE: KeyedVecCache = KeyedVecCache::new();

    CACHE.copy_cached_or_gen(len, saw_count, |len, _seed, saw_count| {
        let mut vals = random_vec(len);
        let chunks_size = len / saw_count.max(1);
        let saw_directions = random_uniform((len / chunks_size) + 1, 0..=1);

        for (i, chunk) in vals.chunks_mut(chunks_size).enumerate() {
            if saw_directions[i] == 0 {
                chunk.sort_unstable();
            } else if saw_directions[i] == 1 {
                chunk.sort_unstable_by_key(|&e| std::cmp::Reverse(e));
            } else {
                unreachable!();
            }
        }

        vals
    })
}

pub fn saw_mixed_range(len: usize, range: std::ops::Range<usize>) -> Vec<i32> {
    //     :.
    // :.  :::.    .::.      .:
    // :::.:::::..::::::..:.:::

    // ascending and descending randomly picked, with length in `range`.

    if len == 0 {
        return Vec::new();
    }

    static CACHE: KeyedVecCache = KeyedVecCache::new();

    CACHE.copy_cached_or_gen(len, range, |len, _seed, range| {
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
                chunk.sort_unstable();
            } else if saw_directions[i] == 1 {
                chunk.sort_unstable_by_key(|&e| std::cmp::Reverse(e));
            } else {
                unreachable!();
            }

            i += 1;
            l += chunk_size;
        }

        vals
    })
}

pub fn pipe_organ(len: usize) -> Vec<i32> {
    //   .:.
    // .:::::.

    static CACHE: VecCache = VecCache::new();

    CACHE.copy_cached_or_gen(len, |len, _seed| {
        let mut vals = random_vec(len);

        let first_half = &mut vals[0..(len / 2)];
        first_half.sort_unstable();

        let second_half = &mut vals[(len / 2)..len];
        second_half.sort_unstable_by_key(|&e| std::cmp::Reverse(e));

        vals
    })
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
    RandomOncePerProcess,
    ExternalOverride,
    RandomEachTime,
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

struct VecCache {
    cache: Mutex<Option<Arc<Vec<i32>>>>,
}

impl VecCache {
    const fn new() -> Self {
        Self {
            cache: Mutex::new(None),
        }
    }

    // Uses fn pointer to avoid accidental captures.
    // Captured values need to be accounted for as part of the key, see KeyedVecCache.
    fn copy_cached_or_gen(&self, len: usize, gen_fn: fn(usize, u64) -> Vec<i32>) -> Vec<i32> {
        let (seed_type, seed_value) = get_or_init_seed_type_and_value();

        if seed_type == SeedType::RandomEachTime {
            return gen_fn(len, seed_value);
        }

        // With a fixed seed, rand will produce the same values in sequence, and lock + memcpy
        // is faster than re-generating them, so we cache previous requests. This is mainly true
        // for debug builds, release and miri see little benefit.

        let mut v_cached_lock = self.cache.lock().unwrap();
        let v_cached = v_cached_lock.get_or_insert_with(Default::default);

        if v_cached.len() >= len {
            // Cheap clone to return control to other threads as fast as possible.
            let v_cached_clone = v_cached.clone();
            drop(v_cached_lock);

            return v_cached_clone[..len].to_vec();
        }

        // We hold the lock while generating the output, this works well when the amount of times
        // other threads are stuck *and* would insert a larger len value is small.
        let v_new = Arc::new(gen_fn(len, seed_value));
        // Cheap clone to return control to other threads as fast as possible.
        *v_cached = v_new.clone();
        drop(v_cached_lock);

        v_new.to_vec()
    }
}

// Because we can't have generics in statics, we manually compute the hash before inserting into the
// HashMap, so to avoid needless double hashing we configure the HashMap with an identity hash
// function.
#[derive(Default)]
struct IdentityHasher(u64);

impl Hasher for IdentityHasher {
    fn write(&mut self, _bytes: &[u8]) {
        unreachable!()
    }

    fn finish(&self) -> u64 {
        self.0
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}

type IdentityBuildHasher = BuildHasherDefault<IdentityHasher>;

struct KeyedVecCache {
    keyed_caches: Mutex<Option<HashMap<u64, Arc<Vec<i32>>, IdentityBuildHasher>>>,
}

impl KeyedVecCache {
    const fn new() -> Self {
        Self {
            keyed_caches: Mutex::new(None),
        }
    }

    fn copy_cached_or_gen<K: Hash>(
        &self,
        len: usize,
        key: K,
        gen_fn: fn(usize, u64, K) -> Vec<i32>,
    ) -> Vec<i32> {
        let (seed_type, seed_value) = get_or_init_seed_type_and_value();

        // Do this early to avoid penalizing the benchmark use-case.
        if seed_type == SeedType::RandomEachTime {
            return gen_fn(len, seed_value, key);
        }

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let key_hash = hasher.finish();

        {
            let keyed_caches_lock = self.keyed_caches.lock().unwrap();

            if let Some(keyed_caches) = keyed_caches_lock.as_ref() {
                if let Some(v_cached) = keyed_caches.get(&key_hash) {
                    if v_cached.len() >= len {
                        // Cheap clone to return control to other threads as fast as possible.
                        let v_cached_arc_clone = v_cached.clone();
                        drop(keyed_caches_lock);

                        return v_cached_arc_clone[..len].to_vec();
                    }
                }
            }

            // Because it's a shared lock drop the lock now and re-acquire later, this might race
            // some work but that's ok.
        }

        let v_new = Arc::new(gen_fn(len, seed_value, key));
        let v_new_clone = v_new.clone();

        {
            let mut keyed_caches_lock = self.keyed_caches.lock().unwrap();
            let v_cached = keyed_caches_lock
                .get_or_insert_with(Default::default)
                .entry(key_hash)
                .or_insert_with(Default::default);

            // Only insert the generated value if no better value was inserted in the meantime by
            // another thread.
            if v_new_clone.len() > v_cached.len() {
                *v_cached = v_new_clone;
            }
        }

        v_new.to_vec()
    }
}

fn random_vec(len: usize) -> Vec<i32> {
    static CACHE: VecCache = VecCache::new();

    CACHE.copy_cached_or_gen(len, |len, seed| {
        let mut rng: StdRng = rand::SeedableRng::seed_from_u64(seed);
        (0..len).map(|_| rng.gen::<i32>()).collect()
    })
}
