use std::cmp::Ordering;
use std::fmt::Debug;
use std::fs;

use sort_comp::fluxsort;
use sort_comp::patterns;

#[cfg(miri)]
const TEST_SIZES: [usize; 23] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 16, 17, 20, 24, 30, 32, 33, 35, 50, 100, 200,
];

#[cfg(not(miri))]
const TEST_SIZES: [usize; 28] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 16, 17, 20, 24, 30, 32, 33, 35, 50, 100, 200, 500, 1_000,
    2_048, 10_000, 100_000,
];

fn sort_comp<T>(v: &mut [T])
where
    T: Ord + Clone + DeepEqual + Debug,
{
    let is_small_test = v.len() <= 100;
    let original_clone = v.to_vec();

    let mut stdlib_sorted_vec = v.to_vec();
    let stdlib_sorted = stdlib_sorted_vec.as_mut_slice();
    stdlib_sorted.sort();

    let fluxsort_sorted = v;
    fluxsort::sort(fluxsort_sorted, |a, b| a.lt(b));

    assert_eq!(stdlib_sorted.len(), fluxsort_sorted.len());

    for (a, b) in stdlib_sorted.iter().zip(fluxsort_sorted.iter()) {
        if !a.deep_equal(b) {
            let seed = patterns::random_init_seed();

            if is_small_test {
                eprintln!("Seed: {seed}");
                eprintln!("Orginal:  {:?}", original_clone);
                eprintln!("Expected: {:?}", stdlib_sorted);
                eprintln!("Got:      {:?}", fluxsort_sorted);
            } else {
                // Large arrays output them as files.
                let original_name = format!("original_{}.txt", seed);
                let std_name = format!("stdlib_sorted_{}.txt", seed);
                let flux_name = format!("fluxsort_sorted_{}.txt", seed);

                fs::write(&original_name, format!("{:?}", original_clone)).unwrap();
                fs::write(&std_name, format!("{:?}", stdlib_sorted)).unwrap();
                fs::write(&flux_name, format!("{:?}", fluxsort_sorted)).unwrap();

                eprintln!(
                    "Failed comparison, see files {original_name}, {std_name}, and {flux_name}"
                );
            }

            panic!("Test assertion failed!")
        }
    }
}

// The idea of this struct is to have something that might look the same, based on the sort property
// but can still be different. This helps test that the stable sort algorithm is actually stable.
#[derive(Clone, Debug, Eq, Ord)]
struct ValueWithExtra {
    key: i32,
    extra: i32,
}

impl PartialOrd for ValueWithExtra {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl PartialEq for ValueWithExtra {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct LargeStackVal {
    val: [u128; 4],
}

trait DeepEqual {
    fn deep_equal(&self, other: &Self) -> bool;
}

impl DeepEqual for () {
    fn deep_equal(&self, _other: &Self) -> bool {
        true
    }
}

impl DeepEqual for i32 {
    fn deep_equal(&self, other: &Self) -> bool {
        self == other
    }
}

impl DeepEqual for String {
    fn deep_equal(&self, other: &Self) -> bool {
        self == other
    }
}

impl DeepEqual for LargeStackVal {
    fn deep_equal(&self, other: &Self) -> bool {
        self == other
    }
}

impl DeepEqual for ValueWithExtra {
    fn deep_equal(&self, other: &Self) -> bool {
        self.key.eq(&other.key) && self.extra.eq(&other.extra)
    }
}

fn test_impl<T: Ord + Clone + DeepEqual + Debug>(pattern_fn: impl Fn(usize) -> Vec<T>) {
    for test_size in TEST_SIZES {
        let mut test_data = pattern_fn(test_size);
        sort_comp(test_data.as_mut_slice());
    }
}

// --- TESTS ---

#[test]
fn basic() {
    sort_comp::<i32>(&mut []);
    sort_comp::<()>(&mut []);
    sort_comp::<()>(&mut [()]);
    sort_comp::<()>(&mut [(), ()]);
    sort_comp::<()>(&mut [(), (), ()]);
    sort_comp(&mut [2, 3]);
    sort_comp(&mut [2, 3, 6]);
    sort_comp(&mut [2, 3, 99, 6]);
    sort_comp(&mut [2, 7709, 400, 90932]);
    sort_comp(&mut [15, -1, 3, -1, -3, -1, 7]);
}

#[test]
fn value_with_extra() {
    let a = ValueWithExtra { key: 6, extra: 9 };
    let b = ValueWithExtra { key: 7, extra: 9 };
    let c = ValueWithExtra { key: 7, extra: 10 };

    assert!(a < b);
    assert!(a < c);
    assert!(b > a);
    assert!(c > a);
    assert!(a != b);
    assert!(a != c);
    assert!(b == c);
    assert!(b == c);

    assert!(!a.deep_equal(&b));
    assert!(!a.deep_equal(&c));
    assert!(!b.deep_equal(&c));
}

#[test]
fn random() {
    test_impl(patterns::random);
}

#[test]
fn all_equal() {
    test_impl(patterns::all_equal);
}

#[test]
fn ascending() {
    test_impl(patterns::ascending);
}

#[test]
fn descending() {
    test_impl(patterns::descending);
}

#[test]
fn ascending_saw() {
    test_impl(|test_size| patterns::ascending_saw(test_size, test_size / 5));
    test_impl(|test_size| patterns::ascending_saw(test_size, test_size / 20));
}

#[test]
fn descending_saw() {
    test_impl(|test_size| patterns::descending_saw(test_size, test_size / 5));
    test_impl(|test_size| patterns::descending_saw(test_size, test_size / 20));
}

#[test]
fn pipe_organ() {
    test_impl(patterns::pipe_organ);
}

#[test]
fn random_duplicates() {
    // This test is designed to stress test stable sorting.
    test_impl(|test_size| {
        let random = patterns::random(test_size);
        let uni = patterns::random_uniform(test_size, 0..(test_size / 10) as i32);

        uni.into_iter()
            .zip(random.into_iter())
            .map(|(key, extra)| ValueWithExtra { key, extra })
            .collect::<Vec<_>>()
    });
}

#[test]
fn random_str() {
    // Much smaller test size to minimize runtime cost, test sorting heap backed values.
    test_impl(|test_size| {
        patterns::random(test_size / 50)
            .into_iter()
            .map(|val| format!("{}", val))
            .collect::<Vec<_>>()
    });
}

#[test]
fn random_large_val() {
    // Much smaller test size to minimize runtime cost, test sorting large stack values.
    test_impl(|test_size| {
        patterns::random(test_size / 50)
            .into_iter()
            .map(|val| {
                let val_abs = val.abs() as u128;
                LargeStackVal {
                    val: [val_abs - 6, val_abs + 3, val_abs - 2, val_abs],
                }
            })
            .collect::<Vec<_>>()
    });
}

#[test]
fn dyn_val() {
    todo!("dyn vals");
}

#[test]
fn comp_panic() {
    todo!("comp panic");
}
