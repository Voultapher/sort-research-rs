#![allow(unused_unsafe)]

/// Sort taken from https://github.com/rust-lang/rust/pull/90545
////////////////////////////////////////////////////////////////////////////////
// Sorting
////////////////////////////////////////////////////////////////////////////////
use std::cmp::Ordering;
use std::mem::{self, size_of};
use std::ptr;

sort_impl!("rust_wpwoodjr_stable");

/// Sorts the slice.
///
/// This sort is stable (i.e., does not reorder equal elements) and *O*(*n* \* log(*n*)) worst-case.
///
/// When applicable, unstable sorting is preferred because it is generally faster than stable
/// sorting and it doesn't allocate auxiliary memory.
/// See [`sort_unstable`](slice::sort_unstable).
///
/// # Current implementation
///
/// The current algorithm is an adaptive, iterative merge sort inspired by
/// [timsort](https://en.wikipedia.org/wiki/Timsort).
/// It is designed to be very fast in cases where the slice is nearly sorted, or consists of
/// two or more sorted sequences concatenated one after another.
///
/// Also, it allocates temporary storage half the size of `self`, but for short slices a
/// non-allocating insertion sort is used instead.
///
/// # Examples
///
/// ```
/// let mut v = [-5, 4, 1, -3, 2];
///
/// v.sort();
/// assert!(v == [-5, -3, 1, 2, 4]);
/// ```
#[inline]
pub fn sort<T>(arr: &mut [T])
where
    T: Ord,
{
    merge_sort(arr, |a, b| a.lt(b));
}

/// Sorts the slice with a comparator function.
///
/// This sort is stable (i.e., does not reorder equal elements) and *O*(*n* \* log(*n*)) worst-case.
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
/// floats.sort_by(|a, b| a.partial_cmp(b).unwrap());
/// assert_eq!(floats, [1.0, 2.0, 3.0, 4.0, 5.0]);
/// ```
///
/// When applicable, unstable sorting is preferred because it is generally faster than stable
/// sorting and it doesn't allocate auxiliary memory.
/// See [`sort_unstable_by`](slice::sort_unstable_by).
///
/// # Current implementation
///
/// The current algorithm is an adaptive, iterative merge sort inspired by
/// [timsort](https://en.wikipedia.org/wiki/Timsort).
/// It is designed to be very fast in cases where the slice is nearly sorted, or consists of
/// two or more sorted sequences concatenated one after another.
///
/// Also, it allocates temporary storage half the size of `self`, but for short slices a
/// non-allocating insertion sort is used instead.
///
/// # Examples
///
/// ```
/// let mut v = [5, 4, 1, 3, 2];
/// v.sort_by(|a, b| a.cmp(b));
/// assert!(v == [1, 2, 3, 4, 5]);
///
/// // reverse sorting
/// v.sort_by(|a, b| b.cmp(a));
/// assert!(v == [5, 4, 3, 2, 1]);
/// ```
#[inline]
pub fn sort_by<T, F>(arr: &mut [T], mut compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    merge_sort(arr, |a, b| compare(a, b) == Ordering::Less);
}

/// Inserts `v[v.len() - 1]` into pre-sorted sequence `v[..v.len() - 1]` so that whole `v[..]` becomes sorted.
///
/// This is the integral subroutine of insertion sort.
#[cfg(not(no_global_oom_handling))]
// benchmarking indicated that inlining makes a substantial improvement, yet only requires a couple of hundred bytes
#[inline(always)]
fn insert_end<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let end = v.len().saturating_sub(1);
    if end > 0 && is_less(&v[end], &v[end - 1]) {
        unsafe {
            // There are three ways to implement insertion here:
            //
            // 1. Swap adjacent elements until the last one gets to its final destination.
            //    However, this way we copy data around more than is necessary. If elements are big
            //    structures (costly to copy), this method will be slow.
            //
            // 2. Iterate until the right place for the last element is found. Then shift the
            //    elements preceeding it to make room for it and finally place it into the
            //    remaining hole. This is a good method.
            //
            // 3. Copy the last element into a temporary variable. Iterate until the right place
            //    for it is found. As we go along, copy every traversed element into the slot
            //    succeeding it. Finally, copy data from the temporary variable into the remaining
            //    hole. This method is very good. Benchmarks demonstrated slightly better
            //    performance than with the 2nd method.
            //
            // All methods were benchmarked, and the 3rd showed best results. So we chose that one.
            let tmp = mem::ManuallyDrop::new(ptr::read(v.get_unchecked(end)));

            // Intermediate state of the insertion process is always tracked by `hole`, which
            // serves two purposes:
            // 1. Protects integrity of `v` from panics in `is_less`.
            // 2. Fills the remaining hole in `v` in the end.
            //
            // Panic safety:
            //
            // If `is_less` panics at any point during the process, `hole` will get dropped and
            // fill the hole in `v` with `tmp`, thus ensuring that `v` still holds every object it
            // initially held exactly once.
            let mut hole = InsertionHole {
                src: &*tmp,
                dest: v.get_unchecked_mut(end - 1),
            };
            ptr::copy_nonoverlapping(hole.dest, v.get_unchecked_mut(end), 1);

            let mut i = end - 1;
            while i > 0 && is_less(&*tmp, v.get_unchecked(i - 1)) {
                hole.dest = v.get_unchecked_mut(i - 1);
                ptr::copy_nonoverlapping(hole.dest, v.get_unchecked_mut(i), 1);
                i -= 1;
            }
            // `hole` gets dropped and thus copies `tmp` into the remaining hole in `v`.
        }
    }

    // When dropped, copies from `src` into `dest`.
    struct InsertionHole<T> {
        src: *const T,
        dest: *mut T,
    }

    impl<T> Drop for InsertionHole<T> {
        fn drop(&mut self) {
            unsafe {
                ptr::copy_nonoverlapping(self.src, self.dest, 1);
            }
        }
    }
}

/// Merges non-decreasing runs `v[..mid]` and `v[mid..]` using `buf` as temporary storage, and
/// stores the result into `v[..]`.
///
/// # Safety
///
/// The two slices must be non-empty and `mid` must be in bounds. Buffer `buf` must be long enough
/// to hold a copy of the shorter slice.
#[allow(unused_unsafe)]
#[cfg(not(no_global_oom_handling))]
unsafe fn merge<T, F>(v: &mut [T], mid: usize, buf: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    let v = v.as_mut_ptr();
    let (v_mid, v_end) = unsafe { (v.add(mid), v.add(len)) };

    // The merge process first copies the shorter run into `buf`. Then it traces the newly copied
    // run and the longer run forwards (or backwards), comparing their next unconsumed elements and
    // copying the lesser (or greater) one into `v`.
    //
    // As soon as the shorter run is fully consumed, the process is done. If the longer run gets
    // consumed first, then we must copy whatever is left of the shorter run into the remaining
    // hole in `v`.
    //
    // Intermediate state of the process is always tracked by `hole`, which serves two purposes:
    // 1. Protects integrity of `v` from panics in `is_less`.
    // 2. Fills the remaining hole in `v` if the longer run gets consumed first.
    //
    // Panic safety:
    //
    // If `is_less` panics at any point during the process, `hole` will get dropped and fill the
    // hole in `v` with the unconsumed range in `buf`, thus ensuring that `v` still holds every
    // object it initially held exactly once.
    let mut hole;

    if mid <= len - mid {
        // The left run is shorter.
        unsafe {
            ptr::copy_nonoverlapping(v, buf, mid);
            hole = MergeHole {
                start: buf,
                end: buf.add(mid),
                dest: v,
            };
        }

        // Initially, these pointers point to the beginnings of their arrays.
        let left = &mut hole.start;
        let mut right = v_mid;
        let out = &mut hole.dest;

        while *left < hole.end && right < v_end {
            // Consume the lesser side.
            // If equal, prefer the left run to maintain stability.
            unsafe {
                let to_copy = if is_less(&*right, &**left) {
                    get_and_increment(&mut right)
                } else {
                    get_and_increment(left)
                };
                ptr::copy_nonoverlapping(to_copy, get_and_increment(out), 1);
            }
        }
    } else {
        // The right run is shorter.
        unsafe {
            ptr::copy_nonoverlapping(v_mid, buf, len - mid);
            hole = MergeHole {
                start: buf,
                end: buf.add(len - mid),
                dest: v_mid,
            };
        }

        // Initially, these pointers point past the ends of their arrays.
        let left = &mut hole.dest;
        let right = &mut hole.end;
        let mut out = v_end;

        while v < *left && buf < *right {
            // Consume the greater side.
            // If equal, prefer the right run to maintain stability.
            unsafe {
                let to_copy = if is_less(&*right.offset(-1), &*left.offset(-1)) {
                    decrement_and_get(left)
                } else {
                    decrement_and_get(right)
                };
                ptr::copy_nonoverlapping(to_copy, decrement_and_get(&mut out), 1);
            }
        }
    }
    // Finally, `hole` gets dropped. If the shorter run was not fully consumed, whatever remains of
    // it will now be copied into the hole in `v`.

    #[inline(always)]
    unsafe fn get_and_increment<T>(ptr: &mut *mut T) -> *mut T {
        let old = *ptr;
        *ptr = unsafe { ptr.offset(1) };
        old
    }

    #[inline(always)]
    unsafe fn decrement_and_get<T>(ptr: &mut *mut T) -> *mut T {
        *ptr = unsafe { ptr.offset(-1) };
        *ptr
    }

    // When dropped, copies the range `start..end` into `dest..`.
    struct MergeHole<T> {
        start: *mut T,
        end: *mut T,
        dest: *mut T,
    }

    impl<T> Drop for MergeHole<T> {
        fn drop(&mut self) {
            // `T` is not a zero-sized type, and these are pointers into a slice's elements.
            unsafe {
                let len = self.end.offset_from_unsigned(self.start);
                ptr::copy_nonoverlapping(self.start, self.dest, len);
            }
        }
    }
}

/// This is a stable two stage merge sort with pre-sorted prefix optimization. The two stages are:
///
/// 1) Top-down recursive depth-first merge, which helps data locality
/// 2) Small slices sort using a fast insertion sort, then merge
///
/// The total running time is *O*(*n* \* log(*n*)) worst-case.
#[cfg(not(no_global_oom_handling))]
fn merge_sort<T, F>(v: &mut [T], mut is_less: F)
where
    F: FnMut(&T, &T) -> bool,
{
    // `gt!` macro centralizes and clarifies the logic.
    // Unchecked array access gives an approximate 20% performance improvement.
    //
    // # Safety
    //
    // `$left` and `$right` must be < `$v.len()`
    macro_rules! gt {
        ($v: ident, $left: expr, $right: expr, $is_less: ident) => {
            // $is_less(&$v[$right], &$v[$left])
            $is_less(unsafe { &$v.get_unchecked($right) }, unsafe {
                &$v.get_unchecked($left)
            })
        };
    }

    // Benchmarking determined these are the best sizes.
    // Recursive merge switches to insertion sort / merge when slice length is <= SMALL_SLICE_LEN*2.
    const SMALL_SLICE_LEN: usize = 10;
    // Slices of up to this length get sorted using insertion sort.
    const MAX_INSERTION: usize = 20;

    // Sorting has no meaningful behavior on zero-sized types.
    if size_of::<T>() == 0 {
        return;
    }

    let len = v.len();
    // Short arrays get sorted in-place via insertion sort to avoid allocations.
    if len <= MAX_INSERTION {
        for i in 1..len {
            insert_end(&mut v[..=i], &mut is_less);
        }
        return;
    }

    // Allocate a buffer to use as scratch memory. We keep the length 0 so we can keep in it
    // shallow copies of the contents of `v` without risking the dtors running on copies if
    // `is_less` panics. When merging two slices, this buffer holds a copy of the right-hand slice,
    // which will always have length at most `(len + 1) / 2`.
    let mut buf = Vec::with_capacity((len + 1) / 2);
    slice_merge_sort(v, 0, buf.as_mut_ptr(), &mut is_less);

    // Do a recursive depth-first merge while slice's length is greater than SMALL_SLICE_LEN*2.
    // Below that length use a combination of insertion sort and merging.
    // For optimization, `sorted` tracks how much of the slice's prefix is already sorted.
    fn slice_merge_sort<T, F>(v: &mut [T], mut sorted: usize, buf_ptr: *mut T, is_less: &mut F)
    where
        F: FnMut(&T, &T) -> bool,
    {
        let len = v.len();
        // find length of sorted prefix
        if sorted == 0 {
            sorted = if len <= 1 {
                len
            } else if gt!(v, 0, 1, is_less) {
                // strictly descending
                let mut i = 2;
                while i < len && gt!(v, i - 1, i, is_less) {
                    i += 1;
                }
                // Reverse the slice so we don't have to sort it later.
                v[..i].reverse();
                i
            } else {
                // ascending
                let mut i = 2;
                while i < len && !gt!(v, i - 1, i, is_less) {
                    i += 1;
                }
                i
            };
        }

        // Do merge sort, using `sorted` to avoid redundant sorting.
        if sorted < len {
            if len <= SMALL_SLICE_LEN + 2 {
                for i in sorted..len {
                    insert_end(&mut v[..=i], is_less);
                }
            } else {
                let mid;
                if len > SMALL_SLICE_LEN * 2 {
                    mid = sorted.max(len / 2);
                    if sorted < mid {
                        slice_merge_sort(&mut v[..mid], sorted, buf_ptr, is_less);
                    }
                    slice_merge_sort(&mut v[mid..], 0, buf_ptr, is_less);
                    if !gt!(v, mid - 1, mid, is_less) {
                        return;
                    } else if gt!(v, 0, len - 1, is_less) {
                        // strictly reverse sorted
                        unsafe {
                            swap_slices(v, mid, buf_ptr);
                        }
                        return;
                    }
                } else {
                    for i in sorted..SMALL_SLICE_LEN {
                        insert_end(&mut v[..=i], is_less);
                    }
                    for i in SMALL_SLICE_LEN + 1..len {
                        insert_end(&mut v[SMALL_SLICE_LEN..=i], is_less);
                    }
                    if !gt!(v, SMALL_SLICE_LEN - 1, SMALL_SLICE_LEN, is_less) {
                        return;
                    }
                    mid = SMALL_SLICE_LEN;
                }
                unsafe {
                    merge(v, mid, buf_ptr, is_less);
                }
            }
        }
    }

    /// swap contents of left-hand and right-hand slices divided at `mid`
    ///
    /// # Safety
    ///
    /// `buf_ptr` must point to a slice of `buf` which is long enough to hold `v[mid..]`
    ///   `v[mid..].len() <= buf.len()` because buf is sized as `(v.len() + 1) / 2' and `mid == sorted.max(v.len() / 2)`
    ///
    /// `mid` must be <= `v.len()`
    ///   `mid <= v.len()` because `swap_slices` is only called when: `sorted < len && mid == sorted.max(len / 2)`
    #[allow(unused_unsafe)]
    unsafe fn swap_slices<T>(v: &mut [T], mid: usize, buf_ptr: *mut T) {
        let rlen = v.len() - mid;
        let v_ptr = v.as_mut_ptr();
        unsafe {
            ptr::copy_nonoverlapping(v_ptr.add(mid), buf_ptr, rlen);
            ptr::copy(v_ptr, v_ptr.add(rlen), mid);
            ptr::copy_nonoverlapping(buf_ptr, v_ptr, rlen);
        }
    }
}
