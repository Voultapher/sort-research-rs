//! Instruction-Parallel-Network Unstable Sort, ipnsort by Lukas Bergdoll
#![allow(incomplete_features, internal_features, stable_features)]
#![feature(
    ptr_sub_ptr,
    auto_traits,
    negative_impls,
    specialization,
    core_intrinsics,
    sized_type_properties,
    const_mut_refs
)]

use core::cmp::Ordering;
use core::intrinsics;
use core::mem::SizedTypeProperties;

mod heapsort;
mod pivot;
mod quicksort;
mod smallsort;

/// Sorts the slice, but might not preserve the order of equal elements.
///
/// This sort is unstable (i.e., may reorder equal elements), in-place
/// (i.e., does not allocate), and *O*(*n* \* log(*n*)) worst-case.
///
/// # Current implementation
///
/// The current algorithm is based on [pattern-defeating quicksort][pdqsort] by Orson Peters,
/// which combines the fast average case of randomized quicksort with the fast worst case of
/// heapsort, while achieving linear time on slices with certain patterns. It uses some
/// randomization to avoid degenerate cases, but with a fixed seed to always provide
/// deterministic behavior.
///
/// It is typically faster than stable sorting, except in a few special cases, e.g., when the
/// slice consists of several concatenated sorted sequences.
///
/// # Examples
///
/// ```
/// let mut v = [-5, 4, 1, -3, 2];
///
/// v.sort_unstable();
/// assert!(v == [-5, -3, 1, 2, 4]);
/// ```
///
/// [pdqsort]: https://github.com/orlp/pdqsort
#[inline(always)]
pub fn sort<T>(arr: &mut [T])
where
    T: Ord,
{
    unstable_sort(arr, |a, b| a.lt(b));
}

/// Sorts the slice with a comparator function, but might not preserve the order of equal
/// elements.
///
/// This sort is unstable (i.e., may reorder equal elements), in-place
/// (i.e., does not allocate), and *O*(*n* \* log(*n*)) worst-case.
///
/// The comparator function must define a total ordering for the elements in the slice. If
/// the ordering is not total, the order of the elements is unspecified. An order is a
/// total order if it is (for all `a`, `b` and `c`):
///
/// * total and antisymmetric: exactly one of `a < b`, `a == b` or `a > b` is true, and
/// * transitive, `a < b` and `b < c` implies `a < c`. The same must hold for both `==` and `>`.
///
/// For example, while [`f64`] doesn't implement [`Ord`] because `NaN != NaN`, we can use
/// `partial_cmp` as our sort function when we know the slice doesn't contain a `NaN`.
///
/// ```
/// let mut floats = [5f64, 4.0, 1.0, 3.0, 2.0];
/// floats.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
/// assert_eq!(floats, [1.0, 2.0, 3.0, 4.0, 5.0]);
/// ```
///
/// # Current implementation
///
/// The current algorithm is based on [pattern-defeating quicksort][pdqsort] by Orson Peters,
/// which combines the fast average case of randomized quicksort with the fast worst case of
/// heapsort, while achieving linear time on slices with certain patterns. It uses some
/// randomization to avoid degenerate cases, but with a fixed seed to always provide
/// deterministic behavior.
///
/// It is typically faster than stable sorting, except in a few special cases, e.g., when the
/// slice consists of several concatenated sorted sequences.
///
/// # Examples
///
/// ```
/// let mut v = [5, 4, 1, 3, 2];
/// v.sort_unstable_by(|a, b| a.cmp(b));
/// assert!(v == [1, 2, 3, 4, 5]);
///
/// // reverse sorting
/// v.sort_unstable_by(|a, b| b.cmp(a));
/// assert!(v == [5, 4, 3, 2, 1]);
/// ```
///
/// [pdqsort]: https://github.com/orlp/pdqsort
#[inline(always)]
pub fn sort_by<T, F>(arr: &mut [T], mut compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    unstable_sort(arr, |a, b| compare(a, b) == Ordering::Less);
}

// --- IMPL ---

/// Sorts `v` using pattern-defeating quicksort, which is *O*(*n* \* log(*n*)) worst-case.
#[inline(always)]
fn unstable_sort<T, F>(v: &mut [T], mut is_less: F)
where
    F: FnMut(&T, &T) -> bool,
{
    // Arrays of zero-sized types are always all-equal, and thus sorted.
    if T::IS_ZST {
        return;
    }

    // Instrumenting the standard library showed that 90+% of the calls to sort
    // by rustc are either of size 0 or 1.
    let len = v.len();
    if intrinsics::likely(len < 2) {
        return;
    }

    // More advanced sorting methods than insertion sort are faster if called in
    // a hot loop for small inputs, but for general-purpose code the small
    // binary size of insertion sort is more important. The instruction cache in
    // modern processors is very valuable, and for a single sort call in general
    // purpose code any gains from an advanced method are cancelled by i-cache
    // misses during the sort, and thrashing the i-cache for surrounding code.
    const MAX_LEN_ALWAYS_INSERTION_SORT: usize = 20;
    if intrinsics::likely(len <= MAX_LEN_ALWAYS_INSERTION_SORT) {
        smallsort::insertion_sort_shift_left(v, 1, &mut is_less);
        return;
    }

    ipnsort(v, &mut is_less);
}

/// TODO explain and link explanation.
#[inline(never)]
fn ipnsort<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    let (run_len, was_reversed) = find_existing_run(v, is_less);

    // SAFETY: find_existing_run promises to return a valid run_len.
    unsafe { intrinsics::assume(run_len <= len) };

    if run_len == len {
        if was_reversed {
            v.reverse();
        }

        // It would be possible to a do in-place merging here for a long existing streak. But that
        // makes the implementation a lot bigger, users can use `slice::sort` for that use-case.
        return;
    }

    // Limit the number of imbalanced partitions to `2 * floor(log2(len))`.
    // The binary OR by one is used to eliminate the zero-check in the logarithm.
    let limit = 2 * (len | 1).ilog2();
    crate::quicksort::quicksort(v, None, limit, is_less);
}

/// Finds a run of sorted elements starting at the beginning of the slice.
///
/// Returns the length of the run, and a bool that is false when the run
/// is ascending, and true if the run strictly descending.
fn find_existing_run<T, F: FnMut(&T, &T) -> bool>(v: &[T], is_less: &mut F) -> (usize, bool) {
    let len = v.len();
    if len < 2 {
        return (len, false);
    }

    // SAFETY: We checked that len >= 2, so 0 and 1 are valid indices.
    // This also means that run_len < len implies run_len and run_len - 1
    // are valid indices as well.
    unsafe {
        let mut run_len = 2;
        let strictly_descending = is_less(v.get_unchecked(1), v.get_unchecked(0));
        if strictly_descending {
            while run_len < len && is_less(v.get_unchecked(run_len), v.get_unchecked(run_len - 1)) {
                run_len += 1;
            }
        } else {
            while run_len < len && !is_less(v.get_unchecked(run_len), v.get_unchecked(run_len - 1))
            {
                run_len += 1;
            }
        }
        (run_len, strictly_descending)
    }
}

// // #[rustc_unsafe_specialization_marker]
// trait Freeze {}

// Can the type have interior mutability, this is checked by testing if T is Freeze. If the type can
// have interior mutability it may alter itself during comparison in a way that must be observed
// after the sort operation concludes. Otherwise a type like Mutex<Option<Box<str>>> could lead to
// double free.
unsafe auto trait Freeze {}

impl<T: ?Sized> !Freeze for core::cell::UnsafeCell<T> {}
unsafe impl<T: ?Sized> Freeze for core::marker::PhantomData<T> {}
unsafe impl<T: ?Sized> Freeze for *const T {}
unsafe impl<T: ?Sized> Freeze for *mut T {}
unsafe impl<T: ?Sized> Freeze for &T {}
unsafe impl<T: ?Sized> Freeze for &mut T {}
