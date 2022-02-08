use std::cell::Cell;
use std::cmp::Ordering;
use std::env;
use std::fmt::Debug;
use std::fs;
use std::io::{self, Write};
use std::panic::{self, AssertUnwindSafe};
use std::rc::Rc;
use std::sync::Mutex;

use sort_comp::ffi_util::{FFIOneKiloByte, FFIString, F128};
use sort_comp::patterns;

use sort_comp::stable::rust_ipn as test_sort;

#[cfg(miri)]
const TEST_SIZES: [usize; 24] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 16, 17, 20, 24, 30, 32, 33, 35, 50, 100, 200, 500,
];

#[cfg(not(miri))]
const TEST_SIZES: [usize; 29] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 16, 17, 20, 24, 30, 32, 33, 35, 50, 100, 200, 500, 1_000,
    2_048, 10_000, 100_000, 1_000_000,
];

fn get_or_init_random_seed() -> u64 {
    static SEED_WRITTEN: Mutex<bool> = Mutex::new(false);
    let seed = patterns::random_init_seed();

    let mut seed_writer = SEED_WRITTEN.lock().unwrap();
    if !*seed_writer {
        // Always write the seed before doing anything to ensure reproducibility of crashes.
        io::stdout()
            .write_all(
                format!(
                    "\nSeed: {seed}\nTesting: {}\n\n",
                    <test_sort::SortImpl as sort_comp::Sort>::name()
                )
                .as_bytes(),
            )
            .unwrap();
        io::stdout().flush().unwrap();

        *seed_writer = true;
    }

    seed
}

fn sort_comp<T>(v: &mut [T])
where
    T: Ord + Clone + Debug,
{
    let seed = get_or_init_random_seed();

    let is_small_test = v.len() <= 100;
    let original_clone = v.to_vec();

    let mut stdlib_sorted_vec = v.to_vec();
    let stdlib_sorted = stdlib_sorted_vec.as_mut_slice();
    stdlib_sorted.sort();

    let testsort_sorted = v;
    test_sort::sort(testsort_sorted);

    assert_eq!(stdlib_sorted.len(), testsort_sorted.len());

    for (a, b) in stdlib_sorted.iter().zip(testsort_sorted.iter()) {
        if a != b {
            if is_small_test {
                eprintln!("Orginal:  {:?}", original_clone);
                eprintln!("Expected: {:?}", stdlib_sorted);
                eprintln!("Got:      {:?}", testsort_sorted);
            } else {
                if env::var("WRITE_LARGE_FAILURE").is_ok() {
                    // Large arrays output them as files.
                    let original_name = format!("original_{}.txt", seed);
                    let std_name = format!("stdlib_sorted_{}.txt", seed);
                    let flux_name = format!("testsort_sorted_{}.txt", seed);

                    fs::write(&original_name, format!("{:?}", original_clone)).unwrap();
                    fs::write(&std_name, format!("{:?}", stdlib_sorted)).unwrap();
                    fs::write(&flux_name, format!("{:?}", testsort_sorted)).unwrap();

                    eprintln!(
                        "Failed comparison, see files {original_name}, {std_name}, and {flux_name}"
                    );
                } else {
                    eprintln!(
                    "Failed comparison, re-run with WRITE_LARGE_FAILURE env var set, to get output."
                );
                }
            }

            panic!("Test assertion failed!")
        }
    }
}

fn test_impl<T: Ord + Clone + Debug>(pattern_fn: impl Fn(usize) -> Vec<T>) {
    for test_size in TEST_SIZES {
        let mut test_data = pattern_fn(test_size);
        sort_comp(test_data.as_mut_slice());
    }
}

fn test_impl_custom(mut test_fn: impl FnMut(usize, fn(usize) -> Vec<i32>)) {
    let test_pattern_fns: Vec<fn(usize) -> Vec<i32>> = vec![
        patterns::random,
        |size| patterns::random_uniform(size, 0..=(((size as f64).log2().round()) as i32) as i32),
        |size| patterns::random_uniform(size, 0..=1 as i32),
        // |size| {
        //     let (len_95p, len_5p) = split_len(size, 95.0);
        //     let v: Vec<i32> = std::iter::repeat(0)
        //         .take(len_95p)
        //         .chain(patterns::random(len_5p))
        //         .collect();

        //     shuffle_vec(v)
        // },
        patterns::ascending,
        patterns::descending,
        |size| patterns::saw_mixed(size, ((size as f64).log2().round()) as usize),
        |size| patterns::saw_mixed(size, (size as f64 / 22.0).round() as usize),
    ];

    for test_pattern_fn in test_pattern_fns {
        for test_size in &TEST_SIZES[..TEST_SIZES.len() - 2] {
            if *test_size < 2 {
                continue;
            }

            test_fn(*test_size, test_pattern_fn);
        }
    }
}

pub trait DynTrait: Debug {
    fn get_val(&self) -> i32;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DynValA {
    value: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DynValB {
    value: i32,
}

impl DynTrait for DynValA {
    fn get_val(&self) -> i32 {
        self.value
    }
}
impl DynTrait for DynValB {
    fn get_val(&self) -> i32 {
        self.value
    }
}

impl PartialOrd for dyn DynTrait {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get_val().partial_cmp(&other.get_val())
    }
}

impl Ord for dyn DynTrait {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for dyn DynTrait {
    fn eq(&self, other: &Self) -> bool {
        self.get_val() == other.get_val()
    }
}

impl Eq for dyn DynTrait {}

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
fn fixed_seed() {
    let fixed_seed_a = patterns::random_init_seed();
    let fixed_seed_b = patterns::random_init_seed();

    assert_eq!(fixed_seed_a, fixed_seed_b);
}

#[test]
fn random() {
    test_impl(patterns::random);
}

#[test]
fn random_4() {
    test_impl(|size| {
        if size > 3 {
            patterns::random_uniform(size, 0..4)
        } else {
            Vec::new()
        }
    });
}

#[test]
fn random_8() {
    test_impl(|size| {
        if size > 3 {
            patterns::random_uniform(size, 0..8)
        } else {
            Vec::new()
        }
    });
}

#[test]
fn random_16() {
    test_impl(|size| {
        if size > 3 {
            patterns::random_uniform(size, 0..16)
        } else {
            Vec::new()
        }
    });
}

#[test]
fn random_256() {
    test_impl(|size| {
        if size > 3 {
            patterns::random_uniform(size, 0..256)
        } else {
            Vec::new()
        }
    });
}

#[test]
fn random_1024() {
    test_impl(|size| {
        if size > 3 {
            patterns::random_uniform(size, 0..1024)
        } else {
            Vec::new()
        }
    });
}

#[test]
fn random_narrow() {
    // Great for debugging.
    test_impl(|size| {
        if size > 3 {
            patterns::random_uniform(size, 0..=(((size as f64).log2().round()) as i32) * 100)
        } else {
            Vec::new()
        }
    });
}

#[test]
fn random_binary() {
    test_impl(|size| patterns::random_uniform(size, 0..=1 as i32));
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
    test_impl(|test_size| {
        patterns::ascending_saw(test_size, ((test_size as f64).log2().round()) as usize)
    });
}

#[test]
fn descending_saw() {
    test_impl(|test_size| {
        patterns::descending_saw(test_size, ((test_size as f64).log2().round()) as usize)
    });
}

#[test]
fn saw_mixed() {
    test_impl(|test_size| {
        patterns::saw_mixed(test_size, ((test_size as f64).log2().round()) as usize)
    });
}

#[test]
fn pipe_organ() {
    test_impl(patterns::pipe_organ);
}

#[test]
fn stability() {
    let _seed = get_or_init_random_seed();

    if <test_sort::SortImpl as sort_comp::Sort>::name().contains("unstable") {
        // It would be great to mark the test as skipped, but that isn't possible as of now.
        return;
    }

    // For cpp_sorts that only support u64 we can pack the two i32 inside a u64.
    fn i32_tup_as_u64(val: (i32, i32)) -> u64 {
        let a_bytes = val.0.to_le_bytes();
        let b_bytes = val.1.to_le_bytes();

        u64::from_le_bytes([a_bytes, b_bytes].concat().try_into().unwrap())
    }

    fn i32_tup_from_u64(val: u64) -> (i32, i32) {
        let bytes = val.to_le_bytes();

        let a = i32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let b = i32::from_le_bytes(bytes[4..8].try_into().unwrap());

        (a, b)
    }

    let large_range = if cfg!(miri) { 100..110 } else { 3000..3010 };
    let rounds = if cfg!(miri) { 1 } else { 10 };

    let rand_vals = patterns::random_uniform(5_000, 0..=9);
    let mut rand_idx = 0;

    for len in (2..55).chain(large_range) {
        for _ in 0..rounds {
            let mut counts = [0; 10];

            // create a vector like [(6, 1), (5, 1), (6, 2), ...],
            // where the first item of each tuple is random, but
            // the second item represents which occurrence of that
            // number this element is, i.e., the second elements
            // will occur in sorted order.
            let orig: Vec<_> = (0..len)
                .map(|_| {
                    let n = rand_vals[rand_idx];
                    rand_idx += 1;
                    if rand_idx >= rand_vals.len() {
                        rand_idx = 0;
                    }

                    counts[n as usize] += 1;
                    i32_tup_as_u64((n, counts[n as usize]))
                })
                .collect();

            let mut v = orig.clone();
            // Only sort on the first element, so an unstable sort
            // may mix up the counts.
            test_sort::sort_by(&mut v, |a_packed, b_packed| {
                let a = i32_tup_from_u64(*a_packed).0;
                let b = i32_tup_from_u64(*b_packed).0;

                a.cmp(&b)
            });

            // This comparison includes the count (the second item
            // of the tuple), so elements with equal first items
            // will need to be ordered with increasing
            // counts... i.e., exactly asserting that this sort is
            // stable.
            assert!(v
                .windows(2)
                .all(|w| i32_tup_from_u64(w[0]) <= i32_tup_from_u64(w[1])));
        }
    }
}

#[test]
fn stability_with_patterns() {
    let _seed = get_or_init_random_seed();

    if <test_sort::SortImpl as sort_comp::Sort>::name().contains("unstable") {
        // It would be great to mark the test as skipped, but that isn't possible as of now.
        return;
    }

    // For cpp_sorts that only support u64 we can pack the two i32 inside a u64.
    fn i32_tup_as_u64(val: (i32, i32)) -> u64 {
        let a_bytes = val.0.to_le_bytes();
        let b_bytes = val.1.to_le_bytes();

        u64::from_le_bytes([a_bytes, b_bytes].concat().try_into().unwrap())
    }

    fn i32_tup_from_u64(val: u64) -> (i32, i32) {
        let bytes = val.to_le_bytes();

        let a = i32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let b = i32::from_le_bytes(bytes[4..8].try_into().unwrap());

        (a, b)
    }

    let test_fn = |test_size: usize, pattern_fn: fn(usize) -> Vec<i32>| {
        let pattern = pattern_fn(test_size);

        let mut counts = [0i32; 128];

        // create a vector like [(6, 1), (5, 1), (6, 2), ...],
        // where the first item of each tuple is random, but
        // the second item represents which occurrence of that
        // number this element is, i.e., the second elements
        // will occur in sorted order.
        let orig: Vec<_> = pattern
            .iter()
            .map(|val| {
                let n = val.saturating_abs() % counts.len() as i32;
                counts[n as usize] += 1;
                i32_tup_as_u64((n, counts[n as usize]))
            })
            .collect();

        let mut v = orig.clone();
        // Only sort on the first element, so an unstable sort
        // may mix up the counts.
        test_sort::sort_by(&mut v, |a_packed, b_packed| {
            let a = i32_tup_from_u64(*a_packed).0;
            let b = i32_tup_from_u64(*b_packed).0;

            a.cmp(&b)
        });

        // This comparison includes the count (the second item
        // of the tuple), so elements with equal first items
        // will need to be ordered with increasing
        // counts... i.e., exactly asserting that this sort is
        // stable.
        assert!(v
            .windows(2)
            .all(|w| i32_tup_from_u64(w[0]) <= i32_tup_from_u64(w[1])));
    };

    test_impl_custom(test_fn);
}

#[test]
fn random_ffi_str() {
    test_impl(|test_size| {
        patterns::random(test_size)
            .into_iter()
            .map(|val| FFIString::new(format!("{:010}", val.saturating_abs())))
            .collect::<Vec<_>>()
    });
}

#[test]
fn random_f128() {
    test_impl(|test_size| {
        patterns::random(test_size)
            .into_iter()
            .map(|val| F128::new(val))
            .collect::<Vec<_>>()
    });
}

#[test]
fn random_str() {
    test_impl(|test_size| {
        patterns::random(test_size)
            .into_iter()
            .map(|val| format!("{}", val))
            .collect::<Vec<_>>()
    });
}

#[test]
fn random_large_val() {
    test_impl(|test_size| {
        if test_size == TEST_SIZES[TEST_SIZES.len() - 1] {
            // That takes too long skip.
            return vec![];
        }

        patterns::random(test_size)
            .into_iter()
            .map(|val| FFIOneKiloByte::new(val))
            .collect::<Vec<_>>()
    });
}

#[test]
fn dyn_val() {
    // Dyn values are fat pointers, something the implementation might have overlooked.
    test_impl(|test_size| {
        patterns::random(test_size)
            .into_iter()
            .map(|val| -> Rc<dyn DynTrait> {
                if val < (i32::MAX / 2) {
                    Rc::new(DynValA { value: val })
                } else {
                    Rc::new(DynValB { value: val })
                }
            })
            .collect::<Vec<Rc<dyn DynTrait>>>()
    });
}

#[test]
fn comp_panic() {
    // Test that sorting upholds panic safety.
    // This means, no non trivial duplicates even if a comparison panics.
    // The invariant being checked is, will miri complain.

    let seed = get_or_init_random_seed();

    let test_fn = |test_size: usize, pattern_fn: fn(usize) -> Vec<i32>| {
        // Needs to be non trivial dtor.
        let mut pattern = pattern_fn(test_size)
            .into_iter()
            .map(|val| vec![val, val, val])
            .collect::<Vec<Vec<i32>>>();

        let val = panic::catch_unwind(AssertUnwindSafe(|| {
            test_sort::sort_by(&mut pattern, |a, b| {
                if a[0].abs() < (i32::MAX / test_size as i32) {
                    panic!(
                        "Explicit panic. Seed: {}. test_size: {}. a: {} b: {}",
                        seed, test_size, a[0], b[0]
                    );
                }

                a[0].cmp(&b[0])
            });

            pattern
                .get(pattern.len().saturating_sub(1))
                .map(|val| val[0])
                .unwrap_or(66)
        }));
        if let Err(err) = val {
            // Side effect.
            println!("{:?}", err);
        }
    };

    test_impl_custom(test_fn);
}

#[cfg(not(miri))]
#[test]
fn observable_is_less_u64() {
    // Technically this is unsound as per Rust semantics, but the only way to do this that works
    // across C FFI. In C and C++ it would be valid to have some trivial POD containing an int that
    // is marked as mutable. Thus allowing member functions to mutate it even though they only have
    // access to a const reference. Now this int could be a pointer that was cleared inside the
    // comparison function, but this clearing is potentially not observable after the sort and it
    // will be freed again. C and C++ have no concept similar to UnsafeCell.
    if <test_sort::SortImpl as sort_comp::Sort>::name().contains("rust_") {
        // It would be great to mark the test as skipped, but that isn't possible as of now.
        return;
    }

    use std::mem;

    let _seed = get_or_init_random_seed();

    // This test, tests that every is_less is actually observable. Ie. this can go wrong if a hole
    // is created using temporary memory and, the whole is used as comparison but not copied back.
    //
    // If this is not upheld a custom type + comparison function could yield UB in otherwise safe
    // code. Eg T == Mutex<Option<Box<str>>> which replaces the pointer with none in the comparison
    // function, which would not be observed in the original slice and would lead to a double free.

    // Pack the comp_count val into a u64, to allow FFI testing, and to ensure that no sort can
    // cheat by treating builtin types differently.
    assert_eq!(mem::size_of::<CompCount>(), mem::size_of::<u64>());
    // Over-aligning is ok.
    assert!(mem::align_of::<CompCount>() <= mem::align_of::<u64>());
    // Ensure it is a small endian system.
    let test_val_u16 = 6043u16;
    assert_eq!(
        unsafe { mem::transmute::<u16, [u8; 2]>(test_val_u16) },
        test_val_u16.to_le_bytes()
    );

    #[derive(PartialEq, Eq, Debug, Clone)]
    #[repr(C)]
    struct CompCount {
        val: i32,
        comp_count: Cell<u32>,
    }

    impl CompCount {
        fn new(val: i32) -> Self {
            Self {
                val,
                comp_count: Cell::new(0),
            }
        }

        fn to_u64(self) -> u64 {
            // SAFETY: See above asserts.
            unsafe { mem::transmute::<Self, u64>(self) }
        }

        fn from_u64(val: &u64) -> &Self {
            // SAFETY: See above asserts.
            unsafe { mem::transmute::<&u64, &Self>(val) }
        }
    }

    let test_fn = |test_size: usize, pattern_fn: fn(usize) -> Vec<i32>| {
        let pattern = pattern_fn(test_size);
        let mut test_input = pattern
            .into_iter()
            .map(|val| CompCount::new(val).to_u64())
            .collect::<Vec<_>>();

        let mut comp_count_gloabl = 0;

        test_sort::sort_by(&mut test_input, |a_u64, b_u64| {
            let a = CompCount::from_u64(a_u64);
            let b = CompCount::from_u64(b_u64);

            a.comp_count.replace(a.comp_count.get() + 1);
            b.comp_count.replace(b.comp_count.get() + 1);
            comp_count_gloabl += 1;

            a.val.cmp(&b.val)
        });

        let total_inner: u64 = test_input
            .iter()
            .map(|c| CompCount::from_u64(c).comp_count.get() as u64)
            .sum();

        assert_eq!(total_inner, comp_count_gloabl * 2);
    };

    test_impl_custom(test_fn);
}

#[test]
fn observable_is_less() {
    let _seed = get_or_init_random_seed();

    // This test, tests that every is_less is actually observable. Ie. this can go wrong if a hole
    // is created using temporary memory and, the whole is used as comparison but not copied back.
    //
    // If this is not upheld a custom type + comparison function could yield UB in otherwise safe
    // code. Eg T == Mutex<Option<Box<str>>> which replaces the pointer with none in the comparison
    // function, which would not be observed in the original slice and would lead to a double free.

    #[derive(PartialEq, Eq, Debug, Clone)]
    #[repr(C)]
    struct CompCount {
        val: i32,
        comp_count: Cell<u32>,
    }

    impl CompCount {
        fn new(val: i32) -> Self {
            Self {
                val,
                comp_count: Cell::new(0),
            }
        }
    }

    let test_fn = |test_size: usize, pattern_fn: fn(usize) -> Vec<i32>| {
        let pattern = pattern_fn(test_size);
        let mut test_input = pattern
            .into_iter()
            .map(|val| CompCount::new(val))
            .collect::<Vec<_>>();

        let mut comp_count_gloabl = 0;

        test_sort::sort_by(&mut test_input, |a, b| {
            a.comp_count.replace(a.comp_count.get() + 1);
            b.comp_count.replace(b.comp_count.get() + 1);
            comp_count_gloabl += 1;

            a.val.cmp(&b.val)
        });

        let total_inner: u64 = test_input.iter().map(|c| c.comp_count.get() as u64).sum();

        assert_eq!(total_inner, comp_count_gloabl * 2);
    };

    test_impl_custom(test_fn);
}

#[test]
fn observable_is_less_mut_ptr() {
    let _seed = get_or_init_random_seed();

    #[derive(PartialEq, Eq, Debug, Clone)]
    struct CompCount {
        val: i32,
        comp_count: u32,
    }

    impl CompCount {
        fn new(val: i32) -> Self {
            Self { val, comp_count: 0 }
        }
    }

    // This test, tests the same as observable_is_less but instead of mutating a Cell like object it
    // mutates *mut pointers.

    let test_fn = |test_size: usize, pattern_fn: fn(usize) -> Vec<i32>| {
        let pattern = pattern_fn(test_size);

        // The sort type T is Copy, yet it still allows mutable access during comparison.
        let mut test_input: Vec<*mut CompCount> = pattern
            .into_iter()
            .map(|val| Box::into_raw(Box::new(CompCount::new(val))))
            .collect::<Vec<_>>();

        let mut comp_count_gloabl = 0;

        test_sort::sort_by(&mut test_input, |a_ptr, b_ptr| {
            let a: &mut CompCount = unsafe { &mut **a_ptr };
            let b: &mut CompCount = unsafe { &mut **b_ptr };

            a.comp_count += 1;
            b.comp_count += 1;
            comp_count_gloabl += 1;

            a.val.cmp(&b.val)
        });

        let total_inner: u64 = test_input
            .iter()
            .map(|c| unsafe { &**c }.comp_count as u64)
            .sum();

        // Drop heap allocated elements.
        for ptr in test_input {
            unsafe {
                drop(Box::from_raw(ptr));
            }
        }

        assert_eq!(total_inner, comp_count_gloabl * 2);
    };

    test_impl_custom(test_fn);
}

fn calc_comps_required<T: Clone>(
    test_data: &[T],
    mut cmp_fn: impl FnMut(&T, &T) -> Ordering,
) -> u32 {
    let mut comp_counter = 0u32;

    let mut test_data_clone = test_data.to_vec();
    test_sort::sort_by(&mut test_data_clone, |a, b| {
        comp_counter += 1;

        cmp_fn(a, b)
    });

    comp_counter
}

#[test]
fn panic_retain_original_set() {
    let _seed = get_or_init_random_seed();

    let test_fn = |test_size: usize, pattern_fn: fn(usize) -> Vec<i32>| {
        let mut test_data = pattern_fn(test_size);

        let sum_before: i64 = test_data.iter().map(|x| *x as i64).sum();

        // Calculate a specific comparison that should panic.
        // Ensure that it can be any of the possible comparisons and that it always panics.
        let required_comps = calc_comps_required(&test_data, |a, b| a.cmp(b));
        let panic_threshold =
            patterns::random_uniform(1, 1..=required_comps as i32)[0] as usize - 1;

        let mut comp_counter = 0;

        let res = panic::catch_unwind(AssertUnwindSafe(|| {
            test_sort::sort_by(&mut test_data, |a, b| {
                if comp_counter == panic_threshold {
                    // Make the panic dependent on the test size and some random factor. We want to
                    // make sure that panicking may also happen when comparing elements a second
                    // time.
                    panic!();
                }
                comp_counter += 1;

                a.cmp(b)
            });
        }));

        assert!(res.is_err());

        // If the sum before and after don't match, it means the set of elements hasn't remained the
        // same.
        let sum_after: i64 = test_data.iter().map(|x| *x as i64).sum();
        assert_eq!(sum_before, sum_after);
    };

    test_impl_custom(test_fn);
}

#[test]
fn panic_observable_is_less() {
    let _seed = get_or_init_random_seed();

    // This test, tests that every is_less is actually observable. Ie. this can go wrong if a hole
    // is created using temporary memory and, the whole is used as comparison but not copied back.
    // This property must also hold if the user provided comparison panics.
    //
    // If this is not upheld a custom type + comparison function could yield UB in otherwise safe
    // code. Eg T == Mutex<Option<Box<str>>> which replaces the pointer with none in the comparison
    // function, which would not be observed in the original slice and would lead to a double free.

    #[derive(PartialEq, Eq, Debug, Clone)]
    #[repr(C)]
    struct CompCount {
        val: i32,
        comp_count: Cell<u32>,
    }

    impl CompCount {
        fn new(val: i32) -> Self {
            Self {
                val,
                comp_count: Cell::new(0),
            }
        }
    }

    let test_fn = |test_size: usize, pattern_fn: fn(usize) -> Vec<i32>| {
        let pattern = pattern_fn(test_size);

        let mut test_input = pattern
            .iter()
            .map(|val| CompCount::new(*val))
            .collect::<Vec<_>>();

        // Calculate a specific comparison that should panic.
        // Ensure that it can be any of the possible comparisons and that it always panics.
        let required_comps = calc_comps_required(&test_input, |a, b| a.val.cmp(&b.val));

        let sum_before: i64 = pattern.iter().map(|x| *x as i64).sum();

        let panic_threshold = patterns::random_uniform(1, 1..=required_comps as i32)[0] as u64 - 1;

        let mut comp_count_gloabl = 0;

        let res = panic::catch_unwind(AssertUnwindSafe(|| {
            test_sort::sort_by(&mut test_input, |a, b| {
                if comp_count_gloabl == panic_threshold {
                    // Make the panic dependent on the test size and some random factor. We want to
                    // make sure that panicking may also happen when comparing elements a second
                    // time.
                    panic!();
                }

                a.comp_count.replace(a.comp_count.get() + 1);
                b.comp_count.replace(b.comp_count.get() + 1);
                comp_count_gloabl += 1;

                a.val.cmp(&b.val)
            });
        }));

        dbg!(comp_count_gloabl);
        assert!(res.is_err());

        let total_inner: u64 = test_input.iter().map(|c| c.comp_count.get() as u64).sum();

        assert_eq!(total_inner, comp_count_gloabl * 2);

        // If the sum before and after don't match, it means the set of elements hasn't remained the
        // same.
        let sum_after: i64 = pattern.iter().map(|x| *x as i64).sum();
        assert_eq!(sum_before, sum_after);
    };

    test_impl_custom(test_fn);
}

#[test]
fn violate_ord_retain_original_set() {
    let _seed = get_or_init_random_seed();

    // A user may implement Ord incorrectly for a type or violate it by calling sort_by with a
    // comparison function that violates Ord with the orderings it returns. Even under such
    // circumstances the input must retain its original set of elements.

    // Ord implies a strict total order. This means that for all a, b and c:
    // A) exactly one of a < b, a == b or a > b is true; and
    // B) < is transitive: a < b and b < c implies a < c. The same must hold for both == and >.

    // Make sure we get a good distribution of random orderings, that are repeatable with the seed.
    // Just using random_uniform with the same size and range will always yield the same value.
    let random_orderings = patterns::random_uniform(5_000, 0..2);

    let get_random_0_1_or_2 = |random_idx: &mut usize| {
        let ridx = *random_idx;
        *random_idx += 1;
        if ridx + 1 == random_orderings.len() {
            *random_idx = 0;
        }

        random_orderings[ridx] as usize
    };

    let mut random_idx_a = 0;
    let mut random_idx_b = 0;
    let mut random_idx_c = 0;

    let mut last_element_a = -1;
    let mut last_element_b = -1;

    let mut rand_counter_b = 0;
    let mut rand_counter_c = 0;

    let mut streak_counter_a = 0;
    let mut streak_counter_b = 0;

    // Examples, a = 3, b = 5, c = 9.
    // Correct Ord -> 10010 | is_less(a, b) is_less(a, a) is_less(b, a) is_less(a, c) is_less(c, a)
    let mut invalid_ord_comp_functions: Vec<Box<dyn FnMut(&i32, &i32) -> Ordering>> = vec![
        Box::new(|_a, _b| -> Ordering {
            // random
            // Eg. is_less(3, 5) == true, is_less(3, 5) == false

            let idx = get_random_0_1_or_2(&mut random_idx_a);
            [Ordering::Less, Ordering::Equal, Ordering::Greater][idx]
        }),
        Box::new(|_a, _b| -> Ordering {
            // everything is less -> 11111
            Ordering::Less
        }),
        Box::new(|_a, _b| -> Ordering {
            // everything is equal -> 00000
            Ordering::Equal
        }),
        Box::new(|_a, _b| -> Ordering {
            // everything is greater -> 00000
            // Eg. is_less(3, 5) == false, is_less(5, 3) == false, is_less(3, 3) == false
            Ordering::Greater
        }),
        Box::new(|a, b| -> Ordering {
            // equal means less else greater -> 01000
            if a == b {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }),
        Box::new(|a, b| -> Ordering {
            // Transitive breaker. remember last element -> 10001
            let lea = last_element_a;
            let leb = last_element_b;

            last_element_a = *a;
            last_element_b = *b;

            if *a == lea && *b != leb {
                b.cmp(a)
            } else {
                a.cmp(b)
            }
        }),
        Box::new(|a, b| -> Ordering {
            // Sampled random 1% of comparisons are reversed.
            rand_counter_b += get_random_0_1_or_2(&mut random_idx_b);
            if rand_counter_b >= 100 {
                rand_counter_b = 0;
                b.cmp(a)
            } else {
                a.cmp(b)
            }
        }),
        Box::new(|a, b| -> Ordering {
            // Sampled random 33% of comparisons are reversed.
            rand_counter_c += get_random_0_1_or_2(&mut random_idx_c);
            if rand_counter_c >= 3 {
                rand_counter_c = 0;
                b.cmp(a)
            } else {
                a.cmp(b)
            }
        }),
        Box::new(|a, b| -> Ordering {
            // STREAK_LEN comparisons yield a.cmp(b) then STREAK_LEN comparisons less. This can
            // discover bugs that neither, random Ord, or just Less or Greater can find. Because it
            // can push a pointer further than expected. Random Ord will average out how far a
            // comparison based pointer travels. Just Less or Greater will be caught by pattern
            // analysis and never enter interesting code.
            const STREAK_LEN: usize = 50;

            streak_counter_a += 1;
            if streak_counter_a <= STREAK_LEN {
                a.cmp(b)
            } else {
                if streak_counter_a == STREAK_LEN * 2 {
                    streak_counter_a = 0;
                }
                Ordering::Less
            }
        }),
        Box::new(|a, b| -> Ordering {
            // See above.
            const STREAK_LEN: usize = 50;

            streak_counter_b += 1;
            if streak_counter_b <= STREAK_LEN {
                a.cmp(b)
            } else {
                if streak_counter_b == STREAK_LEN * 2 {
                    streak_counter_b = 0;
                }
                Ordering::Greater
            }
        }),
    ];

    for comp_func in &mut invalid_ord_comp_functions {
        let test_fn = |test_size: usize, pattern_fn: fn(usize) -> Vec<i32>| {
            let mut test_data = pattern_fn(test_size);
            let sum_before: i64 = test_data.iter().map(|x| *x as i64).sum();

            // It's ok to panic on Ord violation or to complete.
            // In both cases the original elements must still be present.
            let _ = panic::catch_unwind(AssertUnwindSafe(|| {
                test_sort::sort_by(&mut test_data, &mut *comp_func);
            }));

            // If the sum before and after don't match, it means the set of elements hasn't remained the
            // same.
            let sum_after: i64 = test_data.iter().map(|x| *x as i64).sum();
            assert_eq!(sum_before, sum_after);
        };

        test_impl_custom(test_fn);
    }
}

#[test]
fn sort_vs_sort_by() {
    let _seed = get_or_init_random_seed();

    // Ensure that sort and sort_by produce the same result.
    let mut input_normal = [800, 3, -801, 5, -801, -3, 60, 200, 50, 7, 10];
    let expected = [-801, -801, -3, 3, 5, 7, 10, 50, 60, 200, 800];

    let mut input_sort_by = input_normal.to_vec();

    test_sort::sort(&mut input_normal);
    test_sort::sort_by(&mut input_sort_by, |a, b| a.cmp(b));

    assert_eq!(input_normal, expected);
    assert_eq!(input_sort_by, expected);
}

#[test]
fn int_edge() {
    let _seed = get_or_init_random_seed();

    // Ensure that the sort can handle integer edge cases.
    sort_comp(&mut [i32::MIN, i32::MAX]);
    sort_comp(&mut [i32::MAX, i32::MIN]);
    sort_comp(&mut [i32::MIN, 3]);
    sort_comp(&mut [i32::MIN, -3]);
    sort_comp(&mut [i32::MIN, -3, i32::MAX]);
    sort_comp(&mut [i32::MIN, -3, i32::MAX, i32::MIN, 5]);
    sort_comp(&mut [i32::MAX, 3, i32::MIN, 5, i32::MIN, -3, 60, 200, 50, 7, 10]);

    sort_comp(&mut [u64::MIN, u64::MAX]);
    sort_comp(&mut [u64::MAX, u64::MIN]);
    sort_comp(&mut [u64::MIN, 3]);
    sort_comp(&mut [u64::MIN, u64::MAX - 3]);
    sort_comp(&mut [u64::MIN, u64::MAX - 3, u64::MAX]);
    sort_comp(&mut [u64::MIN, u64::MAX - 3, u64::MAX, u64::MIN, 5]);
    sort_comp(&mut [
        u64::MAX,
        3,
        u64::MIN,
        5,
        u64::MIN,
        u64::MAX - 3,
        60,
        200,
        50,
        7,
        10,
    ]);

    let mut large = patterns::random(TEST_SIZES[TEST_SIZES.len() - 2]);
    large.push(i32::MAX);
    large.push(i32::MIN);
    large.push(i32::MAX);
    sort_comp(&mut large);
}
