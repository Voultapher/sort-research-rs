#![allow(unused)]

use std::cmp::Ordering;
use std::mem;
use std::ptr;

#[inline]
pub fn sort<T>(v: &mut [T])
where
    T: Ord,
{
    stable_sort(v, |a, b| a.lt(b));
}

#[inline]
pub fn sort_by<T, F>(v: &mut [T], mut compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    stable_sort(v, |a, b| compare(a, b) == Ordering::Less);
}

#[inline]
pub fn stable_sort<T, F>(v: &mut [T], mut is_less: F)
where
    F: FnMut(&T, &T) -> bool,
{
    if mem::size_of::<T>() == 0 {
        // Sorting has no meaningful behavior on zero-sized types. Do nothing.
        return;
    }

    merge_sort(v, &mut is_less);
}

// Slices of up to this length get sorted using insertion sort.
const MAX_INSERTION: usize = 20;

// Sort a small number of elements as fast as possible, without allocations.
#[inline]
fn sort_small<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    if len < 2 {
        return;
    }

    if T::is_copy() {
        unsafe {
            if len == 2 {
                sort2(v, is_less);
            } else if len == 3 {
                sort3(v, is_less);
            } else if len < 8 {
                sort4(&mut v[..4], is_less);
                insertion_sort_remaining(v, 4, is_less);
            } else if len < 12 {
                sort8(&mut v[..8], is_less);
                insertion_sort_remaining(v, 8, is_less);
            } else if len < 16 {
                sort8(&mut v[..8], is_less);
                sort4(&mut v[8..12], is_less);
                insertion_sort_remaining(v, 8, is_less);
            } else {
                sort16(&mut v[..16], is_less);
                insertion_sort_remaining(v, 16, is_less);
            }
        }
    } else {
        for i in (0..len - 1).rev() {
            // We already checked that len >= 2.
            unsafe {
                insert_head(&mut v[i..], is_less);
            }
        }
    }
}

fn merge_sort<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // Sorting has no meaningful behavior on zero-sized types.
    if mem::size_of::<T>() == 0 {
        return;
    }

    let len = v.len();

    // Short arrays get sorted in-place via insertion sort to avoid allocations.
    if len <= MAX_INSERTION {
        sort_small(v, is_less);
        return;
    }

    // Allocate a buffer to use as scratch memory. We keep the length 0 so we can keep in it
    // shallow copies of the contents of `v` without risking the dtors running on copies if
    // `is_less` panics. When merging two sorted runs, this buffer holds a copy of the shorter run,
    // which will always have length at most `len / 2`.
    let mut buf = Vec::with_capacity(len / 2);

    // In order to identify natural runs in `v`, we traverse it backwards. That might seem like a
    // strange decision, but consider the fact that merges more often go in the opposite direction
    // (forwards). According to benchmarks, merging forwards is slightly faster than merging
    // backwards. To conclude, identifying runs by traversing backwards improves performance.
    let mut runs = vec![];
    let mut end = len;
    while end > 0 {
        // Find the next natural run, and reverse it if it's strictly descending.
        let mut start = end - 1;
        if start > 0 {
            start -= 1;
            unsafe {
                if is_less(v.get_unchecked(start + 1), v.get_unchecked(start)) {
                    while start > 0 && is_less(v.get_unchecked(start), v.get_unchecked(start - 1)) {
                        start -= 1;
                    }
                    v[start..end].reverse();
                } else {
                    while start > 0 && !is_less(v.get_unchecked(start), v.get_unchecked(start - 1))
                    {
                        start -= 1;
                    }
                }
            }
        }

        // SAFETY: end > start.
        start = provide_sorted_batch(v, start, end, is_less);

        // Push this run onto the stack.
        runs.push(Run {
            start,
            len: end - start,
        });
        end = start;

        // Merge some pairs of adjacent runs to satisfy the invariants.
        while let Some(r) = collapse(&runs) {
            let left = runs[r + 1];
            let right = runs[r];
            unsafe {
                merge(
                    &mut v[left.start..right.start + right.len],
                    left.len,
                    buf.as_mut_ptr(),
                    is_less,
                );
            }
            runs[r] = Run {
                start: left.start,
                len: left.len + right.len,
            };
            runs.remove(r + 1);
        }
    }

    // Finally, exactly one run must remain in the stack.
    debug_assert!(runs.len() == 1 && runs[0].start == 0 && runs[0].len == len);

    // Examines the stack of runs and identifies the next pair of runs to merge. More specifically,
    // if `Some(r)` is returned, that means `runs[r]` and `runs[r + 1]` must be merged next. If the
    // algorithm should continue building a new run instead, `None` is returned.
    //
    // TimSort is infamous for its buggy implementations, as described here:
    // http://envisage-project.eu/timsort-specification-and-verification/
    //
    // The gist of the story is: we must enforce the invariants on the top four runs on the stack.
    // Enforcing them on just top three is not sufficient to ensure that the invariants will still
    // hold for *all* runs in the stack.
    //
    // This function correctly checks invariants for the top four runs. Additionally, if the top
    // run starts at index 0, it will always demand a merge operation until the stack is fully
    // collapsed, in order to complete the sort.
    #[inline]
    fn collapse(runs: &[Run]) -> Option<usize> {
        let n = runs.len();
        if n >= 2
            && (runs[n - 1].start == 0
                || runs[n - 2].len <= runs[n - 1].len
                || (n >= 3 && runs[n - 3].len <= runs[n - 2].len + runs[n - 1].len)
                || (n >= 4 && runs[n - 4].len <= runs[n - 3].len + runs[n - 2].len))
        {
            if n >= 3 && runs[n - 3].len < runs[n - 1].len {
                Some(n - 3)
            } else {
                Some(n - 2)
            }
        } else {
            None
        }
    }

    #[derive(Clone, Copy)]
    struct Run {
        len: usize,
        start: usize,
    }
}

/// Takes a range as denoted by start and end, that is already sorted and extends it if necessary
/// with sorts optimized for smaller ranges such as insertion sort.
fn provide_sorted_batch<T, F>(v: &mut [T], mut start: usize, end: usize, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    debug_assert!(end > start);

    const MAX_PRE_SORT16: usize = 8;

    // Testing showed that using MAX_INSERTION here yields the best performance for many types, but
    // incurs more total comparisons. A balance between least comparisons and best performance, as
    // influenced by for example cache locality.
    const MIN_INSERTION_RUN: usize = 10;

    // Insert some more elements into the run if it's too short. Insertion sort is faster than
    // merge sort on short sequences, so this significantly improves performance.
    let start_found = start;
    let start_end_diff = end - start;

    if T::is_copy() && start_end_diff < MAX_PRE_SORT16 && start_found >= 16 {
        unsafe {
            start = start_found.unchecked_sub(16);
            sort16(&mut v[start..start_found], is_less);
        }
        insertion_sort_remaining(&mut v[start..end], 16, is_less);
    } else if start_end_diff < MIN_INSERTION_RUN {
        start = start.saturating_sub(MIN_INSERTION_RUN - start_end_diff);

        for i in (start..start_found).rev() {
            // We ensured that the slice length is always at least 2 long.
            // We know that start_found will be at least one less than end,
            // and the range is exclusive. Which gives us i always <= (end - 2).
            unsafe {
                insert_head(&mut v[i..end], is_less);
            }
        }
    }

    start
}

// When dropped, copies from `src` into `dest`.
struct InsertionHole<T> {
    src: *const T,
    dest: *mut T,
}

impl<T> Drop for InsertionHole<T> {
    fn drop(&mut self) {
        unsafe {
            std::ptr::copy_nonoverlapping(self.src, self.dest, 1);
        }
    }
}

/// Sort v assuming v[..offset] is already sorted.
fn insertion_sort_remaining<T, F>(v: &mut [T], offset: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    // This is a logic but not a safety bug.
    debug_assert!(offset != 0 && offset <= len);

    if len < 2 || offset == 0 {
        return;
    }

    // Shift each element of the unsorted region v[i..] as far left as is needed to make v sorted.
    for i in offset..len {
        // SAFETY: we tested that len >= 2.
        unsafe {
            insert_tail(&mut v[..=i], is_less);
        }
    }
}

/// Inserts `v[v.len() - 1]` into pre-sorted sequence `v[..v.len() - 1]` so that whole `v[..]`
/// becomes sorted.
#[inline]
unsafe fn insert_tail<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    debug_assert!(v.len() >= 2);

    let arr_ptr = v.as_mut_ptr();
    let i = v.len() - 1;

    unsafe {
        // See insert_head which talks about why this approach is beneficial.
        let i_ptr = arr_ptr.add(i);

        // It's important that we use i_ptr here. If this check is positive and we continue,
        // We want to make sure that no other copy of the value was seen by is_less.
        // Otherwise we would have to copy it back.
        if !is_less(&*i_ptr, &*i_ptr.sub(1)) {
            return;
        }

        // It's important, that we use tmp for comparison from now on. As it is the value that
        // will be copied back. And notionally we could have created a divergence if we copy
        // back the wrong value.
        let tmp = mem::ManuallyDrop::new(ptr::read(i_ptr));
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
            dest: i_ptr.sub(1),
        };
        ptr::copy_nonoverlapping(hole.dest, i_ptr, 1);

        // SAFETY: We know i is at least 1.
        for j in (0..(i - 1)).rev() {
            let j_ptr = arr_ptr.add(j);
            if !is_less(&*tmp, &*j_ptr) {
                break;
            }

            ptr::copy_nonoverlapping(j_ptr, hole.dest, 1);
            hole.dest = j_ptr;
        }
        // `hole` gets dropped and thus copies `tmp` into the remaining hole in `v`.
    }
}

/// Inserts `v[0]` into pre-sorted sequence `v[1..]` so that whole `v[..]` becomes sorted.
///
/// This is the integral subroutine of insertion sort.
#[inline]
unsafe fn insert_head<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    debug_assert!(v.len() >= 2);

    if is_less(&v[1], &v[0]) {
        unsafe {
            // There are three ways to implement insertion here:
            //
            // 1. Swap adjacent elements until the first one gets to its final destination.
            //    However, this way we copy data around more than is necessary. If elements are big
            //    structures (costly to copy), this method will be slow.
            //
            // 2. Iterate until the right place for the first element is found. Then shift the
            //    elements succeeding it to make room for it and finally place it into the
            //    remaining hole. This is a good method.
            //
            // 3. Copy the first element into a temporary variable. Iterate until the right place
            //    for it is found. As we go along, copy every traversed element into the slot
            //    preceding it. Finally, copy data from the temporary variable into the remaining
            //    hole. This method is very good. Benchmarks demonstrated slightly better
            //    performance than with the 2nd method.
            //
            // All methods were benchmarked, and the 3rd showed best results. So we chose that one.
            let tmp = mem::ManuallyDrop::new(ptr::read(&v[0]));

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
                dest: &mut v[1],
            };
            ptr::copy_nonoverlapping(&v[1], &mut v[0], 1);

            for i in 2..v.len() {
                if !is_less(&v[i], &*tmp) {
                    break;
                }
                ptr::copy_nonoverlapping(&v[i], &mut v[i - 1], 1);
                hole.dest = &mut v[i];
            }
            // `hole` gets dropped and thus copies `tmp` into the remaining hole in `v`.
        }
    }
}

/// Merges non-decreasing runs `v[..mid]` and `v[mid..]` using `buf` as temporary storage, and
/// stores the result into `v[..]`.
///
/// # Safety
///
/// The two slices must be non-empty and `mid` must be in bounds. Buffer `buf` must be long enough
/// to hold a copy of the shorter slice. Also, `T` must not be a zero-sized type.
unsafe fn merge<T, F>(v: &mut [T], mid: usize, buf: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    let arr_ptr = v.as_mut_ptr();
    let (v_mid, v_end) = unsafe { (arr_ptr.add(mid), arr_ptr.add(len)) };

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
            ptr::copy_nonoverlapping(arr_ptr, buf, mid);
            hole = MergeHole {
                start: buf,
                end: buf.add(mid),
                dest: arr_ptr,
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

        while arr_ptr < *left && buf < *right {
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

    unsafe fn get_and_increment<T>(ptr: &mut *mut T) -> *mut T {
        let old = *ptr;
        *ptr = unsafe { ptr.offset(1) };
        old
    }

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
                let len = self.end.sub_ptr(self.start);
                ptr::copy_nonoverlapping(self.start, self.dest, len);
            }
        }
    }
}

trait IsCopy<T> {
    fn is_copy() -> bool;
}

impl<T> IsCopy<T> for T {
    default fn is_copy() -> bool {
        false
    }
}

impl<T: Copy> IsCopy<T> for T {
    fn is_copy() -> bool {
        true
    }
}

// --- Branchless sorting (less branches not zero) ---

/// Swap value with next value in array pointed to by arr_ptr if the next element is less than a.
#[inline]
pub unsafe fn swap_next_if_less<T, F>(arr_ptr: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: the caller must guarantee that `arr_ptr` and `arr_ptr.add(1)` are valid for writes
    // and properly aligned.
    //
    // PANIC SAFETY: if is_less panics, no scratch memory was created and the slice should still be
    // in a well defined state, without duplicates.

    swap_if_less(arr_ptr, arr_ptr.add(1), is_less);
}

/// Swap two values in array pointed to by a_ptr and b_ptr if b is less than a.
#[inline]
pub unsafe fn swap_if_less<T, F>(a_ptr: *mut T, b_ptr: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: the caller must guarantee that `a_ptr` and `b_ptr` are valid for writes
    // and properly aligned, and part of the same allocation, and do not alias.
    //
    // PANIC SAFETY: if is_less panics, no scratch memory was created and the slice should still be
    // in a well defined state, without duplicates.

    debug_assert!(a_ptr as usize != b_ptr as usize);

    // Important to only swap if it is more and not if it is equal. is_less should return false for
    // equal, so we don't swap.
    let should_swap = is_less(&*b_ptr, &*a_ptr);

    // This is a branchless version of swap if.
    // The equivalent code with a branch would be:
    //
    // if should_swap {
    //     ptr::swap_nonoverlapping(a_ptr, b_ptr, 1);
    // }

    // Give ourselves some scratch space to work with.
    // We do not have to worry about drops: `MaybeUninit` does nothing when dropped.
    let mut tmp = mem::MaybeUninit::<T>::uninit();

    let a_swap_ptr = if should_swap { b_ptr } else { a_ptr };
    let b_swap_ptr = if should_swap { a_ptr } else { b_ptr };

    ptr::copy_nonoverlapping(b_swap_ptr, tmp.as_mut_ptr(), 1);
    ptr::copy(a_swap_ptr, a_ptr, 1);
    ptr::copy_nonoverlapping(tmp.as_ptr(), b_ptr, 1);
}

/// Sort the first 2 elements of v.
unsafe fn sort2<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v is at least len 2.
    debug_assert!(v.len() >= 2);

    swap_next_if_less(v.as_mut_ptr(), is_less);
}

/// Sort the first 3 elements of v.
unsafe fn sort3<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v is at least len 3.
    debug_assert!(v.len() >= 3);

    let arr_ptr = v.as_mut_ptr();
    let x1 = arr_ptr;
    let x2 = arr_ptr.add(1);

    swap_next_if_less(x1, is_less);
    swap_next_if_less(x2, is_less);

    // After two swaps we are here:
    //
    // abc -> ab bc | abc
    // acb -> ac bc | abc
    // bac -> ab bc | abc
    // bca -> bc ac | bac !
    // cab -> ac bc | abc
    // cba -> bc ac | bac !

    // Which means we need to swap again.
    swap_next_if_less(x1, is_less);
}

/// Sort the first 4 elements of v.
unsafe fn sort4<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v is at least len 4.
    debug_assert!(v.len() >= 4);

    let arr_ptr = v.as_mut_ptr();
    let x1 = arr_ptr;
    let x2 = arr_ptr.add(1);
    let x3 = arr_ptr.add(2);

    swap_next_if_less(x1, is_less);
    swap_next_if_less(x3, is_less);

    // PANIC SAFETY: if is_less panics, no scratch memory was created and the slice should still be
    // in a well defined state, without duplicates.
    if is_less(&*x3, &*x2) {
        ptr::swap_nonoverlapping(x2, x3, 1);

        swap_next_if_less(x1, is_less);
        swap_next_if_less(x3, is_less);
        swap_next_if_less(x2, is_less);
    }
}

unsafe fn bitonic_merge8<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: v.len() >= 8
    debug_assert!(v.len() >= 8);

    let arr_ptr = v.as_mut_ptr();

    swap_if_less(arr_ptr.add(0), arr_ptr.add(7), is_less);
    swap_if_less(arr_ptr.add(1), arr_ptr.add(6), is_less);
    swap_if_less(arr_ptr.add(2), arr_ptr.add(5), is_less);
    swap_next_if_less(arr_ptr.add(3), is_less);

    swap_if_less(arr_ptr.add(0), arr_ptr.add(2), is_less);
    swap_if_less(arr_ptr.add(1), arr_ptr.add(3), is_less);

    swap_if_less(arr_ptr.add(4), arr_ptr.add(6), is_less);
    swap_if_less(arr_ptr.add(5), arr_ptr.add(7), is_less);

    swap_next_if_less(arr_ptr.add(0), is_less);
    swap_next_if_less(arr_ptr.add(2), is_less);
    swap_next_if_less(arr_ptr.add(4), is_less);
    swap_next_if_less(arr_ptr.add(6), is_less);
}

unsafe fn bitonic_merge16<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: v.len() >= 16
    debug_assert!(v.len() >= 16);

    let arr_ptr = v.as_mut_ptr();

    swap_if_less(arr_ptr.add(0), arr_ptr.add(15), is_less);
    swap_if_less(arr_ptr.add(1), arr_ptr.add(14), is_less);
    swap_if_less(arr_ptr.add(2), arr_ptr.add(13), is_less);
    swap_if_less(arr_ptr.add(3), arr_ptr.add(12), is_less);
    swap_if_less(arr_ptr.add(4), arr_ptr.add(11), is_less);
    swap_if_less(arr_ptr.add(5), arr_ptr.add(10), is_less);
    swap_if_less(arr_ptr.add(6), arr_ptr.add(9), is_less);
    swap_next_if_less(arr_ptr.add(7), is_less);

    swap_if_less(arr_ptr.add(0), arr_ptr.add(4), is_less);
    swap_if_less(arr_ptr.add(1), arr_ptr.add(5), is_less);
    swap_if_less(arr_ptr.add(2), arr_ptr.add(6), is_less);
    swap_if_less(arr_ptr.add(3), arr_ptr.add(7), is_less);

    swap_if_less(arr_ptr.add(8), arr_ptr.add(12), is_less);
    swap_if_less(arr_ptr.add(9), arr_ptr.add(13), is_less);
    swap_if_less(arr_ptr.add(10), arr_ptr.add(14), is_less);
    swap_if_less(arr_ptr.add(11), arr_ptr.add(15), is_less);

    swap_if_less(arr_ptr.add(0), arr_ptr.add(2), is_less);
    swap_if_less(arr_ptr.add(1), arr_ptr.add(3), is_less);
    swap_if_less(arr_ptr.add(4), arr_ptr.add(6), is_less);
    swap_if_less(arr_ptr.add(5), arr_ptr.add(7), is_less);

    swap_if_less(arr_ptr.add(8), arr_ptr.add(10), is_less);
    swap_if_less(arr_ptr.add(9), arr_ptr.add(11), is_less);
    swap_if_less(arr_ptr.add(12), arr_ptr.add(14), is_less);
    swap_if_less(arr_ptr.add(13), arr_ptr.add(15), is_less);

    swap_next_if_less(arr_ptr.add(0), is_less);
    swap_next_if_less(arr_ptr.add(2), is_less);
    swap_next_if_less(arr_ptr.add(4), is_less);
    swap_next_if_less(arr_ptr.add(6), is_less);
    swap_next_if_less(arr_ptr.add(8), is_less);
    swap_next_if_less(arr_ptr.add(10), is_less);
    swap_next_if_less(arr_ptr.add(12), is_less);
    swap_next_if_less(arr_ptr.add(14), is_less);
}

unsafe fn sort8<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 8.
    debug_assert!(v.len() == 8);

    sort4(v, is_less);
    sort4(&mut v[4..], is_less);

    let arr_ptr = v.as_mut_ptr();

    // Ensure that we do the minimal possible amount of comparisons if the input is already sorted.
    if !is_less(&*arr_ptr.add(4), &*arr_ptr.add(3)) {
        return;
    }

    bitonic_merge8(v, is_less);
}

unsafe fn sort16<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 16.
    debug_assert!(v.len() == 16);

    // Sort the 4 parts of v individually.
    sort4(v, is_less);
    sort4(&mut v[4..], is_less);
    sort4(&mut v[8..], is_less);
    sort4(&mut v[12..], is_less);

    // If all 3 pairs of border elements are sorted, we know the whole 16 elements are now sorted.
    // Doing this check reduces the total comparisons done on average for different input patterns.
    let arr_ptr = v.as_mut_ptr();
    if !is_less(&*arr_ptr.add(4), &*arr_ptr.add(3))
        && !is_less(&*arr_ptr.add(8), &*arr_ptr.add(7))
        && !is_less(&*arr_ptr.add(12), &*arr_ptr.add(11))
    {
        return;
    }

    bitonic_merge8(&mut v[..8], is_less);
    bitonic_merge8(&mut v[8..], is_less);

    bitonic_merge16(v, is_less);
}
