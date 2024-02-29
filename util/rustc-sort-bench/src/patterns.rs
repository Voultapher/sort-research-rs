use rand::prelude::*;

use zipf::ZipfDistribution;

/// Generates `len` non-uniform distributed values in the full `u64` range.
pub fn random(len: usize) -> Vec<u64> {
    //     .
    // : . : :
    // :.:::.::

    let mut rng = thread_rng();

    (0..len).map(|_| rng.gen::<u64>()).collect()
}

/// Generates `len` uniform distributed values in the range `range`.
pub fn random_uniform<R>(size: usize, range: R) -> Vec<u64>
where
    R: Into<rand::distributions::Uniform<u64>>,
{
    // :.:.:.::
    let mut rng = thread_rng();

    // Abstracting over ranges in Rust :(
    let dist: rand::distributions::Uniform<u64> = range.into();

    (0..size).map(|_| dist.sample(&mut rng)).collect()
}

/// Generates `len` non-uniform distributed values in the full `u64` range, where `percent_mid` of
/// it are `u64::MAX / 2`.
pub fn random_x_percent(len: usize, mid_percent: f64) -> Vec<u64> {
    //        .    :
    //  : . ::::.: :  :::.:
    // .:::.::::::::..:::::

    fn split_len(len: usize, part_a_percent: f64) -> (usize, usize) {
        let len_a = ((len as f64 / 100.0) * part_a_percent).round() as usize;
        let len_b = len - len_a;

        (len_a, len_b)
    }

    assert!(mid_percent > 0.0 && mid_percent < 100.0);

    let (len_zero, len_random_p) = split_len(len, 100.0 - mid_percent);

    let mut v: Vec<u64> = std::iter::repeat(u64::MAX / 2)
        .take(len_zero)
        .chain(random(len_random_p))
        .collect();

    let mut rng = thread_rng();
    v.shuffle(&mut rng);

    v
}

/// Generates `len` values that follow the Zipfian distribution.
pub fn random_zipf(len: usize, exponent: f64) -> Vec<u64> {
    // https://en.wikipedia.org/wiki/Zipf's_law

    let mut rng = thread_rng();
    let dist = ZipfDistribution::new(len, exponent).unwrap();

    (0..len).map(|_| dist.sample(&mut rng) as u64).collect()
}

/// Generates `len` non-uniform distributed values in the full `u64` range, where the first
/// `sorted_percent` are already sorted. This simulates adding values to an already sorted slice,
/// and then calling sort again.
pub fn random_sorted(len: usize, sorted_percent: f64) -> Vec<u64> {
    //     .:
    //   .:::. :
    // .::::::.::
    // [----][--]
    //  ^      ^
    //  |      |
    // sorted  |
    //     unsorted

    let mut v = random(len);
    let sorted_len = ((len as f64) * (sorted_percent / 100.0)).round() as usize;

    v[0..sorted_len].sort_unstable();

    v
}
