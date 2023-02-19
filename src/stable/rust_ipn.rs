#![allow(unused)]

//! Instruction-Parallel-Network Stable Sort by Lukas Bergdoll

use std::alloc;
use std::cmp;
use std::cmp::Ordering;
use std::intrinsics;
use std::mem::{self, SizedTypeProperties};
use std::ptr;

sort_impl!("rust_ipn_stable");

#[inline(always)]
pub fn sort<T>(v: &mut [T])
where
    T: Ord,
{
    stable_sort(v, |a, b| a.cmp(b));
}

#[inline(always)]
pub fn sort_by<T, F>(v: &mut [T], compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    stable_sort(v, compare);
}

////////////////////////////////////////////////////////////////////////////////
// Sorting
////////////////////////////////////////////////////////////////////////////////

#[inline(always)]
fn stable_sort<T, F>(v: &mut [T], mut compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    if T::IS_ZST {
        // Sorting has no meaningful behavior on zero-sized types. Do nothing.
        return;
    }

    let elem_alloc_fn = |len: usize| -> *mut T {
        // SAFETY: Creating the layout is safe as long as merge_sort never calls this with len >
        // v.len(). Alloc in general will only be used as 'shadow-region' to store temporary swap
        // elements.
        unsafe { alloc::alloc(alloc::Layout::array::<T>(len).unwrap_unchecked()) as *mut T }
    };

    let elem_dealloc_fn = |buf_ptr: *mut T, len: usize| {
        // SAFETY: Creating the layout is safe as long as merge_sort never calls this with len >
        // v.len(). The caller must ensure that buf_ptr was created by elem_alloc_fn with the same
        // len.
        unsafe {
            alloc::dealloc(
                buf_ptr as *mut u8,
                alloc::Layout::array::<T>(len).unwrap_unchecked(),
            );
        }
    };

    let run_alloc_fn = |len: usize| -> *mut TimSortRun {
        // SAFETY: Creating the layout is safe as long as merge_sort never calls this with an
        // obscene length or 0.
        unsafe {
            alloc::alloc(alloc::Layout::array::<TimSortRun>(len).unwrap_unchecked())
                as *mut TimSortRun
        }
    };

    let run_dealloc_fn = |buf_ptr: *mut TimSortRun, len: usize| {
        // SAFETY: The caller must ensure that buf_ptr was created by elem_alloc_fn with the same
        // len.
        unsafe {
            alloc::dealloc(
                buf_ptr as *mut u8,
                alloc::Layout::array::<TimSortRun>(len).unwrap_unchecked(),
            );
        }
    };

    merge_sort(
        v,
        &mut compare,
        elem_alloc_fn,
        elem_dealloc_fn,
        run_alloc_fn,
        run_dealloc_fn,
    );
}

unsafe fn maybe_uninit_from_buf<'a, T>(
    ptr: &'a *mut T,
    len: usize,
) -> &'a mut [mem::MaybeUninit<T>] {
    &mut *ptr::slice_from_raw_parts_mut(*ptr as *mut mem::MaybeUninit<T>, len)
}

fn unpack_maybe_unit_slice<T>(buf: &mut [mem::MaybeUninit<T>]) -> (*mut T, usize) {
    let buf_ptr = mem::MaybeUninit::slice_as_mut_ptr(buf);
    let buf_len = buf.len();

    (buf_ptr, buf_len)
}

#[inline(always)]
pub fn merge_sort<T, CmpF, ElemAllocF, ElemDeallocF, RunAllocF, RunDeallocF>(
    v: &mut [T],
    compare: &mut CmpF,
    elem_alloc_fn: ElemAllocF,
    elem_dealloc_fn: ElemDeallocF,
    run_alloc_fn: RunAllocF,
    run_dealloc_fn: RunDeallocF,
) where
    CmpF: FnMut(&T, &T) -> Ordering,
    ElemAllocF: Fn(usize) -> *mut T,
    ElemDeallocF: Fn(*mut T, usize),
    RunAllocF: Fn(usize) -> *mut TimSortRun + Copy,
    RunDeallocF: Fn(*mut TimSortRun, usize) + Copy,
{
    // The caller should have already checked that.
    debug_assert!(!T::IS_ZST);

    let len = v.len();

    // This path is critical for very small inputs. Always pick insertion sort for these inputs,
    // without any other analysis. This is perf critical for small inputs, in cold code.
    if intrinsics::likely(len <= max_len_always_insertion_sort::<T>()) {
        if intrinsics::likely(len >= 2) {
            insertion_sort_shift_left(v, 1, &mut |a, b| compare(a, b) == Ordering::Less);
        }

        return;
    }

    if sort_small_stable_with_analysis(v, &mut |a, b| compare(a, b) == Ordering::Less) {
        return;
    }

    // Experiments with stack allocation for small inputs showed worse performance.
    // May depend on the platform.
    let buf_len_wish = len;
    let buf_len_fallback_min = len / 2;
    let buf = BufGuard::new(
        buf_len_wish,
        buf_len_fallback_min,
        elem_alloc_fn,
        elem_dealloc_fn,
    );
    let buf_ptr = buf.buf_ptr.as_ptr();
    let buf_len = buf.capacity;

    // SAFETY: BufGuard has to report the correct ptr and len.
    let buf_m = unsafe { maybe_uninit_from_buf(&buf_ptr, buf_len) };

    merge_sort_impl(v, compare, buf_m, run_alloc_fn, run_dealloc_fn);

    // Extremely basic versions of Vec.
    // Their use is super limited and by having the code here, it allows reuse between the sort
    // implementations.
    struct BufGuard<T, ElemDeallocF>
    where
        ElemDeallocF: Fn(*mut T, usize),
    {
        buf_ptr: ptr::NonNull<T>,
        capacity: usize,
        elem_dealloc_fn: ElemDeallocF,
    }

    impl<T, ElemDeallocF> BufGuard<T, ElemDeallocF>
    where
        ElemDeallocF: Fn(*mut T, usize),
    {
        fn new<ElemAllocF>(
            len_wish: usize,
            len_fallback_min: usize,
            elem_alloc_fn: ElemAllocF,
            elem_dealloc_fn: ElemDeallocF,
        ) -> Self
        where
            ElemAllocF: Fn(usize) -> *mut T,
        {
            let mut buf_ptr = elem_alloc_fn(len_wish);
            let mut capacity = len_wish;

            // There are overcommit style global allocators, but on such systems chances are half
            // the length is gonna be a problem too.
            if buf_ptr.is_null() {
                buf_ptr = elem_alloc_fn(len_fallback_min);
                capacity = len_fallback_min;

                if buf_ptr.is_null() {
                    // Maybe fall back to in-place stable sort?
                    panic!("Unable to allocate memory for sort");
                }
            }

            Self {
                buf_ptr: ptr::NonNull::new(buf_ptr).unwrap(),
                capacity,
                elem_dealloc_fn,
            }
        }
    }

    impl<T, ElemDeallocF> Drop for BufGuard<T, ElemDeallocF>
    where
        ElemDeallocF: Fn(*mut T, usize),
    {
        fn drop(&mut self) {
            (self.elem_dealloc_fn)(self.buf_ptr.as_ptr(), self.capacity);
        }
    }
}

/// This will only be called for inputs size > 20, so it's ok to have this not-inlined. This way we
/// can get more of the performance critical stuff, for small inputs inlined directly into the
/// caller.
#[inline(never)]
pub fn merge_sort_impl<T, CmpF, RunAllocF, RunDeallocF>(
    v: &mut [T],
    compare: &mut CmpF,
    buf: &mut [mem::MaybeUninit<T>],
    run_alloc_fn: RunAllocF,
    run_dealloc_fn: RunDeallocF,
) where
    CmpF: FnMut(&T, &T) -> Ordering,
    RunAllocF: Fn(usize) -> *mut TimSortRun + Copy,
    RunDeallocF: Fn(*mut TimSortRun, usize) + Copy,
{
    // The caller should have already checked that.
    debug_assert!(!T::IS_ZST);

    let len = v.len();

    let (buf_ptr, buf_len) = unpack_maybe_unit_slice(buf);

    // Limit the possibility of doing consecutive ineffective partitions.
    let min_good_partiton_len = len / 16;
    let min_re_probe_distance = cmp::max(256, min_good_partiton_len);

    let mut runs = TimSortRunVec::new(&run_alloc_fn, &run_dealloc_fn);
    let mut equal_runs = TimSortRunVec::new(&run_alloc_fn, &run_dealloc_fn);

    let mut start = 0;
    let mut end = 0;
    let mut run_end = len;

    // The logic for this get's hairy if buf_len is not the full length of v. If the fallback
    // allocation size was used, common value filtering doesn't work anymore. Plus if memory is that
    // constrained the extra allocations needed for equal_runs may be problematic too.
    let mut next_probe_spot = if buf_len == len { 0 } else { usize::MAX };

    // Scan forward. Memory pre-fetching prefers forward scanning vs backwards scanning, and the
    // code-gen is usually better. For the most sensitive types such as integers, these are merged
    // bidirectionally at once. So there is no benefit in scanning backwards.
    while end < run_end {
        let local_v = &mut v[start..run_end];

        let probe_for_common = start >= next_probe_spot;

        // Probe for common value with priority over streak analysis.
        if probe_for_common {
            let mut equal_count = 0;

            if let Some(common_idx) =
                probe_for_common_val(local_v, &mut |a, b| compare(a, b) == Ordering::Equal)
            {
                // SAFETY: Caller must ensure if probe_for_common is set to true that `buf` is valid for
                // `v.len()` writes.
                equal_count = unsafe {
                    partition_equal_stable(local_v, common_idx, buf_ptr, &mut |a, b| {
                        compare(a, b) == Ordering::Equal
                    })
                };
            }

            if equal_count >= min_good_partiton_len {
                run_end -= equal_count;
                equal_runs.push(TimSortRun {
                    start: run_end,
                    len: equal_count,
                });
            } else {
                // Avoid re-probing the same area again and again if probing failed or was of low
                // quality.
                next_probe_spot = start + min_re_probe_distance;
            }

            // It's important that this can reach the collapse, otherwise it might miss the end
            // condition for the non equal runs.
        } else {
            let (mut local_end, was_reversed) =
                find_streak(local_v, &mut |a, b| compare(a, b) == Ordering::Less);
            if was_reversed {
                local_v[..local_end].reverse();
            }

            // Insert some more elements into the run if it's too short. Insertion sort is faster than
            // merge sort on short sequences, so this significantly improves performance.
            local_end = provide_sorted_batch(local_v, 0, local_end, &mut |a, b| {
                compare(a, b) == Ordering::Less
            });

            end = start + local_end;

            // Push this run onto the stack.
            runs.push(TimSortRun {
                start,
                len: local_end,
            });
            start = end;
        }

        // Merge some pairs of adjacent runs to satisfy the invariants.
        while let Some(r) = collapse(runs.as_slice(), run_end) {
            let left = runs[r];
            let right = runs[r + 1];
            let merge_slice = &mut v[left.start..right.start + right.len];

            // check_vec = unsafe { mem::transmute::<&[T], &[DebugT]>(merge_slice).to_vec() };

            // SAFETY: TODO
            unsafe {
                merge(merge_slice, left.len, buf_ptr, buf_len, &mut |a, b| {
                    compare(a, b) == Ordering::Less
                });
            }
            runs[r + 1] = TimSortRun {
                start: left.start,
                len: left.len + right.len,
            };
            runs.remove(r);

            // check_vec.sort();
            // let x = unsafe { mem::transmute::<&[T], &[DebugT]>(merge_slice) };
            // assert_eq!(x, check_vec);
        }
    }

    if equal_runs.len != 0 {
        // Now take care of all the fully equal runs that were put at the end of v.

        // SAFETY: TODO
        unsafe {
            merge_equal_runs(
                v,
                equal_runs.as_mut_slice(),
                buf_ptr,
                buf_len,
                &mut |a, b| compare(a, b) == Ordering::Less,
            );
        }

        debug_assert!(runs.len() <= 1);

        if run_end > 0 {
            // SAFETY: TODO
            unsafe {
                merge(v, run_end, buf_ptr, buf_len, &mut |a, b| {
                    compare(a, b) == Ordering::Less
                });
            }
        }
    } else {
        // Finally, exactly one run must remain in the stack.
        debug_assert!(runs.len() == 1 && runs[0].start == 0 && runs[0].len == len);
    }
}

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
// run ends at stop, it will always demand a merge operation until the stack is fully
// collapsed, in order to complete the sort.
#[inline(always)]
fn collapse(runs: &[TimSortRun], stop: usize) -> Option<usize> {
    let n = runs.len();
    if n >= 2
        && (runs[n - 1].start + runs[n - 1].len == stop
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

/// Internal type used by merge_sort.
#[derive(Clone, Copy, Debug)]
pub struct TimSortRun {
    len: usize,
    start: usize,
    // all_equal: bool,
}

struct TimSortRunVec<RunAllocF, RunDeallocF>
where
    RunAllocF: Fn(usize) -> *mut TimSortRun,
    RunDeallocF: Fn(*mut TimSortRun, usize),
{
    buf_ptr: ptr::NonNull<TimSortRun>,
    capacity: usize,
    len: usize,
    run_alloc_fn: RunAllocF,
    run_dealloc_fn: RunDeallocF,
}

impl<RunAllocF, RunDeallocF> TimSortRunVec<RunAllocF, RunDeallocF>
where
    RunAllocF: Fn(usize) -> *mut TimSortRun,
    RunDeallocF: Fn(*mut TimSortRun, usize),
{
    fn new(run_alloc_fn: RunAllocF, run_dealloc_fn: RunDeallocF) -> Self {
        // Most slices can be sorted with at most 16 runs in-flight.
        const START_RUN_CAPACITY: usize = 16;

        Self {
            buf_ptr: ptr::NonNull::new(run_alloc_fn(START_RUN_CAPACITY)).unwrap(),
            capacity: START_RUN_CAPACITY,
            len: 0,
            run_alloc_fn,
            run_dealloc_fn,
        }
    }

    fn push(&mut self, val: TimSortRun) {
        if self.len == self.capacity {
            let old_capacity = self.capacity;
            let old_buf_ptr = self.buf_ptr.as_ptr();

            self.capacity = self.capacity * 2;
            self.buf_ptr = ptr::NonNull::new((self.run_alloc_fn)(self.capacity)).unwrap();

            // SAFETY: buf_ptr new and old were correctly allocated and old_buf_ptr has
            // old_capacity valid elements.
            unsafe {
                ptr::copy_nonoverlapping(old_buf_ptr, self.buf_ptr.as_ptr(), old_capacity);
            }

            (self.run_dealloc_fn)(old_buf_ptr, old_capacity);
        }

        // SAFETY: The invariant was just checked.
        unsafe {
            self.buf_ptr.as_ptr().add(self.len).write(val);
        }
        self.len += 1;
    }

    fn remove(&mut self, index: usize) {
        if index >= self.len {
            panic!("Index out of bounds");
        }

        // SAFETY: buf_ptr needs to be valid and len invariant upheld.
        unsafe {
            // the place we are taking from.
            let ptr = self.buf_ptr.as_ptr().add(index);

            // Shift everything down to fill in that spot.
            ptr::copy(ptr.add(1), ptr, self.len - index - 1);
        }
        self.len -= 1;
    }

    fn as_slice(&self) -> &[TimSortRun] {
        // SAFETY: Safe as long as buf_ptr is valid and len invariant was upheld.
        unsafe { &*ptr::slice_from_raw_parts(self.buf_ptr.as_ptr(), self.len) }
    }

    fn as_mut_slice(&mut self) -> &mut [TimSortRun] {
        // SAFETY: Safe as long as buf_ptr is valid and len invariant was upheld.
        unsafe { &mut *ptr::slice_from_raw_parts_mut(self.buf_ptr.as_ptr(), self.len) }
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl<RunAllocF, RunDeallocF> core::ops::Index<usize> for TimSortRunVec<RunAllocF, RunDeallocF>
where
    RunAllocF: Fn(usize) -> *mut TimSortRun,
    RunDeallocF: Fn(*mut TimSortRun, usize),
{
    type Output = TimSortRun;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.len {
            // SAFETY: buf_ptr and len invariant must be upheld.
            unsafe {
                return &*(self.buf_ptr.as_ptr().add(index));
            }
        }

        panic!("Index out of bounds");
    }
}

impl<RunAllocF, RunDeallocF> core::ops::IndexMut<usize> for TimSortRunVec<RunAllocF, RunDeallocF>
where
    RunAllocF: Fn(usize) -> *mut TimSortRun,
    RunDeallocF: Fn(*mut TimSortRun, usize),
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index < self.len {
            // SAFETY: buf_ptr and len invariant must be upheld.
            unsafe {
                return &mut *(self.buf_ptr.as_ptr().add(index));
            }
        }

        panic!("Index out of bounds");
    }
}

impl<RunAllocF, RunDeallocF> Drop for TimSortRunVec<RunAllocF, RunDeallocF>
where
    RunAllocF: Fn(usize) -> *mut TimSortRun,
    RunDeallocF: Fn(*mut TimSortRun, usize),
{
    fn drop(&mut self) {
        // As long as TimSortRun is Copy we don't need to drop them individually but just the
        // whole allocation.
        (self.run_dealloc_fn)(self.buf_ptr.as_ptr(), self.capacity);
    }
}

/// Finds a streak of presorted elements starting at the beginning of the slice. Returns the first
/// value that is not part of said streak, and a bool denoting wether the streak was reversed.
/// Streaks can be increasing or decreasing.
fn find_streak<T, F>(v: &[T], is_less: &mut F) -> (usize, bool)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    if len < 2 {
        return (len, false);
    }

    let mut end = 2;

    // SAFETY: See below specific.
    unsafe {
        // SAFETY: We checked that len >= 2, so 0 and 1 are valid indices.
        let assume_reverse = is_less(v.get_unchecked(1), v.get_unchecked(0));

        // SAFETY: We know end >= 2 and check end < len.
        // From that follows that accessing v at end and end - 1 is safe.
        if assume_reverse {
            while end < len && is_less(v.get_unchecked(end), v.get_unchecked(end - 1)) {
                end += 1;
            }

            (end, true)
        } else {
            while end < len && !is_less(v.get_unchecked(end), v.get_unchecked(end - 1)) {
                end += 1;
            }
            (end, false)
        }
    }
}

/// Keep this of the critical inlined path. This is only called for non-tiny slices.
#[inline(never)]
fn sort_small_stable_with_analysis<T, F>(v: &mut [T], is_less: &mut F) -> bool
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    assert!(len > max_len_always_insertion_sort::<T>());

    if len > max_len_small_sort_stable::<T>() {
        return false;
    }

    // For larger inputs it's worth it to check if they are already ascending or descending.
    let (streak_end, was_reversed) = find_streak(v, is_less);

    if !qualifies_for_stable_sort_network::<T>() || (len - streak_end) <= cmp::max(len / 2, 8) {
        if was_reversed {
            v[..streak_end].reverse();
        }

        insertion_sort_shift_left(v, streak_end, is_less);
        return true;
    }

    let even_len = len - (len % 2 != 0) as usize;
    let len_div_2 = even_len / 2;

    // This logic is only works if max_len_always_insertion_sort is at least 15.
    // Otherwise the slice operation for the second sort8 will fail.
    let pre_sorted = if len < 32 {
        sort8_stable(&mut v[0..8], is_less);
        sort8_stable(&mut v[len_div_2..(len_div_2 + 8)], is_less);

        8
    } else {
        sort16_stable(&mut v[0..16], is_less);
        sort16_stable(&mut v[len_div_2..(len_div_2 + 16)], is_less);

        16
    };

    insertion_sort_shift_left(&mut v[0..len_div_2], pre_sorted, is_less);
    insertion_sort_shift_left(&mut v[len_div_2..], pre_sorted, is_less);

    // Unfortunately max_len_small_sort_stable can't be currently be const, this is a workaround.
    const SWAP_LEN: usize = 40;
    debug_assert!(SWAP_LEN == max_len_small_sort_stable::<T>());

    let mut swap = mem::MaybeUninit::<[T; SWAP_LEN]>::uninit();
    let swap_ptr = swap.as_mut_ptr() as *mut T;

    // SAFETY: We checked that T is Copy and thus observation safe. Should is_less panic v was not
    // modified in bi_directional_merge_even and retains it's original input. swap and v must not
    // alias and swap has v.len() space.
    unsafe {
        bi_directional_merge_even(&mut v[..even_len], swap_ptr, is_less);
        ptr::copy_nonoverlapping(swap_ptr, v.as_mut_ptr(), even_len);
    }

    if len != even_len {
        // SAFETY: We know len >= 2.
        unsafe {
            insert_tail(v, is_less);
        }
    }

    true
}

// Slices of up to this length get sorted using optimized sorting for small slices.
fn max_len_small_sort_stable<T>() -> usize {
    if qualifies_for_stable_sort_network::<T>() {
        40
    } else {
        20
    }
}

// Slices of up to this length always get sorted with insertion sort, directly inlined as part of
// the hot path.
const fn max_len_always_insertion_sort<T>() -> usize {
    15
}

/// Takes a range as denoted by start and end, that is already sorted and extends it to the right if
/// necessary with sorts optimized for smaller ranges such as insertion sort.
fn provide_sorted_batch<T, F>(v: &mut [T], start: usize, mut end: usize, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    assert!(end >= start && end <= len);

    // Insert some more elements into the run if it's too short. Insertion sort is faster than
    // merge sort on short sequences, so this significantly improves performance.
    let start_end_diff = end - start;

    // This value is a balance between least comparisons and best performance, as
    // influenced by for example cache locality.
    const MAX_IGNORE_PRE_SORTED: usize = 6;
    const FAST_SORT_SIZE: usize = 32;

    // Reduce the border conditions where new runs are created that don't fit FAST_SORT_SIZE.
    let min_insertion_run = if qualifies_for_stable_sort_network::<T>() {
        20
    } else {
        10
    };

    if qualifies_for_stable_sort_network::<T>()
        && (start + FAST_SORT_SIZE) <= len
        && start_end_diff <= MAX_IGNORE_PRE_SORTED
    {
        // For random inputs on average how many elements are naturally already sorted
        // (start_end_diff) will be relatively small. And it's faster to avoid a merge operation
        // between the newly sorted elements by the sort network and the already sorted
        // elements. Instead just run the sort network and ignore the already sorted streak.
        //
        // Note, this optimization significantly reduces comparison count, versus just always using
        // insertion_sort_shift_left. Insertion sort is faster than calling merge here, and this is
        // yet faster starting at FAST_SORT_SIZE 20.
        end = start + FAST_SORT_SIZE;

        // Use a straight-line sorting network here instead of some hybrid network with early
        // exit. If the input is already sorted the previous adaptive analysis path of TimSort
        // ought to have found it. So we prefer minimizing the total amount of comparisons,
        // which are user provided and may be of arbitrary cost.
        sort32_stable(&mut v[start..(start + FAST_SORT_SIZE)], is_less);
    } else if start_end_diff < min_insertion_run && end < len {
        // v[start_found..end] are elements that are already sorted in the input. We want to extend
        // the sorted region to the left, so we push up min_insertion_run - 1 to the right. Which is
        // more efficient that trying to push those already sorted elements to the left.
        end = cmp::min(start + min_insertion_run, len);
        let presorted_start = cmp::max(start_end_diff, 1);

        insertion_sort_shift_left(&mut v[start..end], presorted_start, is_less);
    }

    end
}

/// Merges non-decreasing runs `v[..mid]` and `v[mid..]` using `buf` as temporary storage, and
/// stores the result into `v[..]`.
///
/// # Safety
///
/// The two slices must be non-empty and `mid` must be in bounds. Buffer `buf` must be long enough
/// to hold a copy of the shorter slice. Also, `T` must not be a zero-sized type.
///
/// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
/// performance impact.
#[inline(never)]
unsafe fn merge_fallback<T, F>(v: &mut [T], mid: usize, buf: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    let v = v.as_mut_ptr();

    // SAFETY: mid and len must be in-bounds of v.
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

        // SAFETY: buf must have enough capacity for `v[..mid]`.
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

            // SAFETY: left and right must be valid and part of v same for out.
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

        // SAFETY: buf must have enough capacity for `v[mid..]`.
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

            // SAFETY: left and right must be valid and part of v same for out.
            unsafe {
                let to_copy = if is_less(&*right.sub(1), &*left.sub(1)) {
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

        // SAFETY: ptr.add(1) must still be a valid pointer and part of `v`.
        *ptr = unsafe { ptr.add(1) };
        old
    }

    unsafe fn decrement_and_get<T>(ptr: &mut *mut T) -> *mut T {
        // SAFETY: ptr.sub(1) must still be a valid pointer and part of `v`.
        *ptr = unsafe { ptr.sub(1) };
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
            // SAFETY: `T` is not a zero-sized type, and these are pointers into a slice's elements.
            unsafe {
                let len = self.end.sub_ptr(self.start);
                ptr::copy_nonoverlapping(self.start, self.dest, len);
            }
        }
    }
}

/// Merges non-decreasing runs `v[..mid]` and `v[mid..]` using `buf` as temporary storage, and
/// stores the result into `v[..]`.
///
/// # Safety
///
/// Buffer as pointed to by `buf_ptr` must have space for `buf_len` writes. And must not alias `v`.
#[inline(always)]
unsafe fn merge<T, F>(v: &mut [T], mid: usize, buf_ptr: *mut T, buf_len: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    assert!(mid > 0 && mid < len && buf_len >= (cmp::min(mid, len - mid)));
    debug_assert!(!buf_ptr.is_null());

    // SAFETY: We checked that the two slices must be non-empty and `mid` must be in bounds. The
    // caller has to guarantee that Buffer `buf` must be long enough to hold a copy of the shorter
    // slice. Also, `T` must not be a zero-sized type. We checked that T is observation safe. Should
    // is_less panic v was not modified in bi_directional_merge and retains it's original input.
    // buf_ptr and v must not alias and swap has v.len() space.
    unsafe {
        if !has_direct_iterior_mutability::<T>() && len <= buf_len {
            bi_directional_merge(v, mid, buf_ptr, is_less);
            ptr::copy_nonoverlapping(buf_ptr, v.as_mut_ptr(), len);
        } else {
            merge_fallback(v, mid, buf_ptr, is_less);
        }
    }
}

// #[rustc_unsafe_specialization_marker]
trait IsCopyMarker {}

impl<T: Copy> IsCopyMarker for T {}

trait IsCopy {
    fn is_copy() -> bool;
}

impl<T> IsCopy for T {
    default fn is_copy() -> bool {
        false
    }
}

impl<T: IsCopyMarker> IsCopy for T {
    fn is_copy() -> bool {
        true
    }
}

#[inline(always)]
const fn is_cheap_to_move<T>() -> bool {
    // This is a heuristic, and as such it will guess wrong from time to time. The two parts broken
    // down:
    //
    // - Type size: Large types are more expensive to move and the time won avoiding branches can be
    //              offset by the increased cost of moving the values.
    //
    // In contrast to stable sort, using sorting networks here, allows to do fewer comparisons.
    mem::size_of::<T>() <= mem::size_of::<[usize; 4]>()
}

// I would like to make this a const fn.
#[inline(always)]
fn has_direct_iterior_mutability<T>() -> bool {
    // - Can the type have interior mutability, this is checked by testing if T is Copy.
    //   If the type can have interior mutability it may alter itself during comparison in a way
    //   that must be observed after the sort operation concludes.
    //   Otherwise a type like Mutex<Option<Box<str>>> could lead to double free.
    //   FIXME use proper abstraction
    !T::is_copy()
}

#[inline(always)]
fn qualifies_for_stable_sort_network<T>() -> bool {
    // This is only a heuristic but, generally for expensive to compare types, it's not worth it to
    // use the stable sorting-network. Which is great at extracting instruction-level parallelism
    // (ILP) for types like integers, but not for a complex type with indirection.
    is_cheap_to_move::<T>() && T::is_copy()
}

#[inline(always)]
unsafe fn merge_up<T, F>(
    mut src_left: *const T,
    mut src_right: *const T,
    mut dest_ptr: *mut T,
    is_less: &mut F,
) -> (*const T, *const T, *mut T)
where
    F: FnMut(&T, &T) -> bool,
{
    // This is a branchless merge utility function.
    // The equivalent code with a branch would be:
    //
    // if !is_less(&*src_right, &*src_left) {
    //     ptr::copy_nonoverlapping(src_left, dest_ptr, 1);
    //     src_left = src_left.wrapping_add(1);
    // } else {
    //     ptr::copy_nonoverlapping(src_right, dest_ptr, 1);
    //     src_right = src_right.wrapping_add(1);
    // }
    // dest_ptr = dest_ptr.add(1);

    // SAFETY: The caller must guarantee that `src_left`, `src_right` are valid to read and
    // `dest_ptr` is valid to write, while not aliasing.
    unsafe {
        let is_l = !is_less(&*src_right, &*src_left);
        let copy_ptr = if is_l { src_left } else { src_right };
        ptr::copy_nonoverlapping(copy_ptr, dest_ptr, 1);
        src_right = src_right.wrapping_add(!is_l as usize);
        src_left = src_left.wrapping_add(is_l as usize);
        dest_ptr = dest_ptr.add(1);
    }

    (src_left, src_right, dest_ptr)
}

#[inline(always)]
unsafe fn merge_down<T, F>(
    mut src_left: *const T,
    mut src_right: *const T,
    mut dest_ptr: *mut T,
    is_less: &mut F,
) -> (*const T, *const T, *mut T)
where
    F: FnMut(&T, &T) -> bool,
{
    // This is a branchless merge utility function.
    // The equivalent code with a branch would be:
    //
    // if !is_less(&*src_right, &*src_left) {
    //     ptr::copy_nonoverlapping(src_right, dest_ptr, 1);
    //     src_right = src_right.wrapping_sub(1);
    // } else {
    //     ptr::copy_nonoverlapping(src_left, dest_ptr, 1);
    //     src_left = src_left.wrapping_sub(1);
    // }
    // dest_ptr = dest_ptr.sub(1);

    // SAFETY: The caller must guarantee that `src_left`, `src_right` are valid to read and
    // `dest_ptr` is valid to write, while not aliasing.
    unsafe {
        let is_l = !is_less(&*src_right, &*src_left);
        let copy_ptr = if is_l { src_right } else { src_left };
        ptr::copy_nonoverlapping(copy_ptr, dest_ptr, 1);
        src_right = src_right.wrapping_sub(is_l as usize);
        src_left = src_left.wrapping_sub(!is_l as usize);
        dest_ptr = dest_ptr.sub(1);
    }

    (src_left, src_right, dest_ptr)
}

/// Merge v assuming the len is even and v[..len / 2] and v[len / 2..] are sorted.
///
/// Original idea for bi-directional merging by Igor van den Hoven (quadsort), adapted to only use
/// merge up and down. In contrast to the original parity_merge function, it performs 2 writes
/// instead of 4 per iteration. Ord violation detection was added.
pub unsafe fn bi_directional_merge_even<T, F>(v: &[T], dest_ptr: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: the caller must guarantee that `dest_ptr` is valid for v.len() writes.
    // Also `v.as_ptr` and `dest_ptr` must not alias.
    //
    // The caller must guarantee that T cannot modify itself inside is_less.
    // merge_up and merge_down read left and right pointers and potentially modify the stack value
    // they point to, if T has interior mutability. This may leave one or two potential writes to
    // the stack value un-observed when dest is copied onto of src.

    // It helps to visualize the merge:
    //
    // Initial:
    //
    //  |ptr_data (in dest)
    //  |ptr_left           |ptr_right
    //  v                   v
    // [xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx]
    //                     ^                   ^
    //                     |t_ptr_left         |t_ptr_right
    //                                         |t_ptr_data (in dest)
    //
    // After:
    //
    //                      |ptr_data (in dest)
    //        |ptr_left     |           |ptr_right
    //        v             v           v
    // [xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx]
    //       ^             ^           ^
    //       |t_ptr_left   |           |t_ptr_right
    //                     |t_ptr_data (in dest)
    //
    //
    // Note, the pointers that have been written, are now one past where they were read and
    // copied. written == incremented or decremented + copy to dest.

    assert!(!has_direct_iterior_mutability::<T>());

    let len = v.len();
    let src_ptr = v.as_ptr();

    let len_div_2 = len / 2;

    // SAFETY: No matter what the result of the user-provided comparison function is, all 4 read
    // pointers will always be in-bounds. Writing `ptr_data` and `t_ptr_data` will always be in
    // bounds if the caller guarantees that `dest_ptr` is valid for `v.len()` writes.
    unsafe {
        let mut ptr_left = src_ptr;
        let mut ptr_right = src_ptr.wrapping_add(len_div_2);
        let mut ptr_data = dest_ptr;

        let mut t_ptr_left = src_ptr.wrapping_add(len_div_2 - 1);
        let mut t_ptr_right = src_ptr.wrapping_add(len - 1);
        let mut t_ptr_data = dest_ptr.wrapping_add(len - 1);

        for _ in 0..len_div_2 {
            (ptr_left, ptr_right, ptr_data) = merge_up(ptr_left, ptr_right, ptr_data, is_less);
            (t_ptr_left, t_ptr_right, t_ptr_data) =
                merge_down(t_ptr_left, t_ptr_right, t_ptr_data, is_less);
        }

        let left_diff = (ptr_left as usize).wrapping_sub(t_ptr_left as usize);
        let right_diff = (ptr_right as usize).wrapping_sub(t_ptr_right as usize);

        if !(left_diff == mem::size_of::<T>() && right_diff == mem::size_of::<T>()) {
            panic_on_ord_violation();
        }
    }
}

/// Merge v assuming v[..mid] and v[mid..] are already sorted.
#[inline(never)]
pub unsafe fn bi_directional_merge<T, F>(v: &[T], mid: usize, dest_ptr: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: the caller must guarantee that `dest_ptr` is valid for v.len() writes.
    // Also `v.as_ptr` and `dest_ptr` must not alias.
    //
    // The caller must guarantee that T cannot modify itself inside is_less.
    // merge_up and merge_down read left and right pointers and potentially modify the stack value
    // they point to, if T has interior mutability. This may leave one or two potential writes to
    // the stack value un-observed when dest is copied onto of src.
    unsafe {
        let len = v.len();
        let src_ptr = v.as_ptr();

        debug_assert!(!has_direct_iterior_mutability::<T>());

        // The original idea for bi-directional merging comes from Igor van den Hoven (quadsort), the
        // code was adapted to only perform 2 writes per loop iteration instead of 4. And it was adapted
        // to support un-balanced merges. is branchless (jump-less) regarding the comparison result,
        // this function can exploit significantly more instruction-level parallelism.

        // It helps to visualize the merge:
        //
        //                        mid
        //  |left_ptr     right_ptr|
        //  v                      v
        // [xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx]
        //                        ^                                    ^
        //                        |t_left_ptr               t_right_ptr|
        //
        // If there is no ord violation left_ptr and t_left_ptr should meet somewhere inside the
        // left side. And right_ptr t_right_ptr somewhere in the right side.
        // Note, left_ptr and right_ptr can only grow (move to the right) and,
        // t_left_ptr and t_right_ptr can only shrink (move to the left).
        //
        // Along with each loop iteration of merge_up and merge_down ptr_data will grow by 1 and
        // t_ptr_data shrink by 1.
        // During the merge buffer looks like this:
        // [xxxxxxxxxxxxxuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuxxxxxxxxxxxxx]
        // Where x is values that have been written and u are potentially uninitialized memory.

        let mut ptr_left = src_ptr;
        let mut ptr_right = src_ptr.wrapping_add(mid);
        let mut ptr_data = dest_ptr;

        let mut t_ptr_left = src_ptr.wrapping_add(mid - 1);
        let mut t_ptr_right = src_ptr.wrapping_add(len - 1);
        let mut t_ptr_data = dest_ptr.wrapping_add(len - 1);

        let left_side_shorter = mid < len - mid;
        let shorter_side = if left_side_shorter { mid } else { len - mid };
        let longer_side = len - shorter_side;

        // Even if Ord is implemented incorrectly, ptr_left and ptr_right combined can only move at most
        // shorter_side steps up, and t_ptr_left and t_ptr_right at most shorter_side steps down. If mid
        // is inbounds and shorter_side as promised at least 1, no matter what comparison results were
        // returned, all reads will be in-bounds. The range for each pointer is:
        // - ptr_left    [0..shorter_side)
        // - ptr_right   [mid..(mid + shorter_side))
        // - t_ptr_left  [(mid - 1 - shorter_side)..(mid - 1))
        // - t_ptr_right [(len - 1 - shorter_side)..(len - 1)
        for _ in 0..shorter_side {
            (ptr_left, ptr_right, ptr_data) = merge_up(ptr_left, ptr_right, ptr_data, is_less);
            (t_ptr_left, t_ptr_right, t_ptr_data) =
                merge_down(t_ptr_left, t_ptr_right, t_ptr_data, is_less);
        }

        let calc_ptr_diff = |ptr, base_ptr| (ptr as usize).wrapping_sub(base_ptr as usize);

        if shorter_side != longer_side {
            // Bounds check every of the 4 pointers that can be read for each loop-iteration. Either it
            // completes normally and one of the two merge attempts is done, or Ord was violated and one
            // or multiple pointers would now be out-of-bounds.
            while ptr_left <= t_ptr_left
                && t_ptr_left >= src_ptr
                && ptr_right <= t_ptr_right
                && t_ptr_right >= src_ptr
            {
                (ptr_left, ptr_right, ptr_data) = merge_up(ptr_left, ptr_right, ptr_data, is_less);
                (t_ptr_left, t_ptr_right, t_ptr_data) =
                    merge_down(t_ptr_left, t_ptr_right, t_ptr_data, is_less);
            }

            let mid_ptr = src_ptr.add(mid);
            let end_ptr = src_ptr.add(len);

            let left_ptr_done = calc_ptr_diff(ptr_left, t_ptr_left) == mem::size_of::<T>();
            let right_ptr_done = calc_ptr_diff(ptr_right, t_ptr_right) == mem::size_of::<T>();

            if !left_ptr_done && !right_ptr_done {
                panic_on_ord_violation();
            }

            if !left_ptr_done {
                // Be vigilant and check everything that could go wrong.
                // t_ptr_left must be within the left side and larger or equal to ptr_left.
                if !(t_ptr_data >= ptr_data && t_ptr_left < mid_ptr && t_ptr_left >= ptr_left) {
                    panic_on_ord_violation();
                }

                let buf_rest_len = t_ptr_data.sub_ptr(ptr_data) + 1;
                let copy_len = t_ptr_left.sub_ptr(ptr_left) + 1;
                assert!(copy_len == buf_rest_len);
                ptr::copy_nonoverlapping(ptr_left, ptr_data, copy_len);
                ptr_left = ptr_left.add(copy_len);
            } else if !right_ptr_done {
                // t_ptr_right must be within the right side and larger or equal to ptr_right.
                if !(t_ptr_data >= ptr_data && t_ptr_right < end_ptr && t_ptr_right >= ptr_right) {
                    panic_on_ord_violation();
                }

                let buf_rest_len = t_ptr_data.sub_ptr(ptr_data) + 1;
                let copy_len = t_ptr_right.sub_ptr(ptr_right) + 1;
                assert!(copy_len == buf_rest_len);
                ptr::copy_nonoverlapping(ptr_right, ptr_data, copy_len);
                ptr_right = ptr_right.add(copy_len);
            }
        }

        let left_diff = calc_ptr_diff(ptr_left, t_ptr_left);
        let right_diff = calc_ptr_diff(ptr_right, t_ptr_right);

        if !(left_diff == mem::size_of::<T>() && right_diff == mem::size_of::<T>()) {
            panic_on_ord_violation();
        }
    }
}

// When dropped, copies from `src` into `dest`.
struct InsertionHole<T> {
    src: *const T,
    dest: *mut T,
}

impl<T> Drop for InsertionHole<T> {
    fn drop(&mut self) {
        // SAFETY: This is a helper class. Please refer to its usage for correctness. Namely, one
        // must be sure that `src` and `dst` does not overlap as required by
        // `ptr::copy_nonoverlapping` and are both valid for writes.
        unsafe {
            ptr::copy_nonoverlapping(self.src, self.dest, 1);
        }
    }
}

/// Inserts `v[v.len() - 1]` into pre-sorted sequence `v[..v.len() - 1]` so that whole `v[..]`
/// becomes sorted.
unsafe fn insert_tail<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    debug_assert!(v.len() >= 2);

    let arr_ptr = v.as_mut_ptr();
    let i = v.len() - 1;

    // SAFETY: caller must ensure v is at least len 2.
    unsafe {
        // See insert_head which talks about why this approach is beneficial.
        let i_ptr = arr_ptr.add(i);

        // It's important that we use i_ptr here. If this check is positive and we continue,
        // We want to make sure that no other copy of the value was seen by is_less.
        // Otherwise we would have to copy it back.
        if is_less(&*i_ptr, &*i_ptr.sub(1)) {
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
}

/// Sort `v` assuming `v[..offset]` is already sorted.
fn insertion_sort_shift_left<T, F>(v: &mut [T], offset: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    // Using assert here improves performance.
    assert!(offset != 0 && offset <= len);

    // Shift each element of the unsorted region v[i..] as far left as is needed to make v sorted.
    for i in offset..len {
        // SAFETY: we tested that `offset` must be at least 1, so this loop is only entered if len
        // >= 2. The range is exclusive and we know `i` must be at least 1 so this slice has at
        // >least len 2.
        unsafe {
            insert_tail(&mut v[..=i], is_less);
        }
    }
}

/// Probe `v` and try to find a recurring value. Starting at `v[start..]`.
/// Returns the index of said element if it has a high confidence that it is recurring.
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn probe_for_common_val<T, F>(v: &[T], is_equal: &mut F) -> Option<usize>
where
    F: FnMut(&T, &T) -> bool,
{
    // Two-stage probing to improve accuracy. Reducing false positives and false negatives.
    const PROBE_REGION_SIZE: usize = 256;
    const INITAL_PROBE_SET_SIZE: usize = 8;
    const VERIFY_PROBE_STEPS: usize = 32;
    const INITIAL_PROBE_OFFSET: usize = PROBE_REGION_SIZE / 2;
    const MIN_HIT_COUNT: u8 = 2;

    let len = v.len();

    if len < PROBE_REGION_SIZE {
        return None;
    }

    let mut inital_match_counts = mem::MaybeUninit::<[u8; INITAL_PROBE_SET_SIZE]>::uninit();
    let inital_match_counts_ptr = inital_match_counts.as_mut_ptr() as *mut u8;

    // Ideally the optimizer will unroll this.
    for i in 0..INITAL_PROBE_SET_SIZE {
        // SAFETY: INITAL_PROBE_SET_SIZE is used as array size and INITIAL_PROBE_OFFSET + i is
        // withing the checked bounds of `v`.
        unsafe {
            *inital_match_counts_ptr.add(i) = is_equal(
                v.get_unchecked(i),
                v.get_unchecked(INITIAL_PROBE_OFFSET + i),
            ) as u8;
        }
    }

    // SAFETY: We initialized all the values in the loop above.
    let inital_match_counts = unsafe { inital_match_counts.assume_init() };
    let inital_match_counts_u64 = u64::from_ne_bytes(inital_match_counts);

    if inital_match_counts_u64 == 0 {
        return None;
    }

    // At least one element was found again, do deeper probing to find out if is common.
    let best_idx = if cfg!(target_endian = "little") {
        ((inital_match_counts_u64.trailing_zeros() + 1) / 8) as usize
    } else {
        ((inital_match_counts_u64.leading_zeros() + 1) / 8) as usize
    };

    let candidate = &v[best_idx];
    // Check it against locations that have not yet been checked.
    // Already checked:
    // v[<0..8> <IPO..IPO+8>]

    let mut count = 0;
    for i in (PROBE_REGION_SIZE - VERIFY_PROBE_STEPS)..PROBE_REGION_SIZE {
        // SAFETY: Access happens inside checked PROBE_REGION_SIZE.
        let elem = unsafe { v.get_unchecked(i) };
        count += is_equal(candidate, elem) as u8;

        if i == (VERIFY_PROBE_STEPS / 2) && count == 0 {
            break;
        }

        if count >= MIN_HIT_COUNT {
            return Some(best_idx);
        }
    }

    None
}

/// Partition `v` into elements that are not equal to `v[pivot_pos]` followed by elements equal to
/// `v[pivot_pos]`. Relative position of `v[pivot_pos]` is maintained.
///
/// Returns the number of element equal to `v[pivot_pos]`.
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
unsafe fn partition_equal_stable<T, F>(
    v: &mut [T],
    pivot_pos: usize,
    buf: *mut T,
    is_equal: &mut F,
) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    let arr_ptr = v.as_mut_ptr();

    // SAFETY: The caller must ensure `buf` is valid for `v.len()` writes.
    // See specific comments below.
    unsafe {
        let pivot_val = mem::ManuallyDrop::new(ptr::read(&v[pivot_pos]));
        // It's crucial that pivot_hole will be copied back to the input if any comparison in the
        // loop panics. Because it could have changed due to interior mutability.
        let pivot_hole = InsertionHole {
            src: &*pivot_val,
            dest: arr_ptr.add(pivot_pos),
        };

        let mut swap_ptr_l = buf;
        let mut swap_ptr_r = buf.add(len.saturating_sub(1));
        let mut pivot_partioned_ptr = ptr::null_mut();

        for i in 0..len {
            // This should only happen once and be branch that can be predicted very well.
            if i == pivot_pos {
                // Technically we are leaving a hole in buf here, but we don't overwrite `v` until
                // all comparisons have been done. So this should be fine. We patch it up later to
                // make sure that a unique observation path happened for `pivot_val`. If we just
                // write the value as pointed to by `elem_ptr` into `buf` as it was in the input
                // slice `v` we would risk that the call to `is_equal` modifies the value pointed to
                // by `elem_ptr`. This could be UB for types such as `Mutex<Option<Box<String>>>`
                // where during the comparison it replaces the box with None, leading to double
                // free. As the value written back into `v` from `buf` did not observe that
                // modification.
                pivot_partioned_ptr = swap_ptr_r;
                swap_ptr_r = swap_ptr_r.sub(1);
                continue;
            }

            let elem_ptr = arr_ptr.add(i);

            let is_eq = is_equal(&*elem_ptr, &pivot_val);

            ptr::copy_nonoverlapping(elem_ptr, swap_ptr_l, 1);
            ptr::copy_nonoverlapping(elem_ptr, swap_ptr_r, 1);

            swap_ptr_l = swap_ptr_l.wrapping_add(!is_eq as usize);
            swap_ptr_r = swap_ptr_r.wrapping_sub(is_eq as usize);
        }

        debug_assert!((swap_ptr_l as usize).abs_diff(swap_ptr_r as usize) == mem::size_of::<T>());

        // SAFETY: swap now contains all elements, `swap[..l_count]` has the elements that are not
        // equal and swap[l_count..]` all the elements that are equal but reversed. All comparisons
        // have been done now, if is_less would have panicked v would have stayed untouched.
        let l_count = swap_ptr_l.sub_ptr(buf);
        let r_count = len - l_count;

        // Copy pivot_val into it's correct position.
        mem::forget(pivot_hole);
        ptr::copy_nonoverlapping(&*pivot_val, pivot_partioned_ptr, 1);

        // Now that swap has the correct order overwrite arr_ptr.
        let arr_ptr = v.as_mut_ptr();

        // Copy all the elements that were not equal directly from swap to v.
        ptr::copy_nonoverlapping(buf, arr_ptr, l_count);

        // Copy the elements that were equal or more from the buf into v and reverse them.
        let rev_buf_ptr = buf.add(len - 1);
        for i in 0..r_count {
            ptr::copy_nonoverlapping(rev_buf_ptr.sub(i), arr_ptr.add(l_count + i), 1);
        }

        r_count
    }
}

/// Merge the `runs` of `v` assuming they all each contain only equal elements and each run has a
/// unique value. The runs must be contiguous in `v`.
///
/// SAFETY: The caller must ensure that buf_ptr is valid to write for the total length of the runs.
unsafe fn merge_equal_runs<T, F>(
    v: &mut [T],
    runs: &mut [TimSortRun],
    buf_ptr: *mut T,
    buf_len: usize,
    is_less: &mut F,
) where
    F: FnMut(&T, &T) -> bool,
{
    if runs.len() <= 1 {
        return;
    }

    // The way it was added this is the left-most run.
    // Create a copy for later.
    let left_most_run: TimSortRun = runs[runs.len() - 1];
    // The partitioned runs are always put at the end.
    let total_run_len = v.len() - left_most_run.start;
    debug_assert!(total_run_len <= buf_len);

    // Figure out the required oder by sorting via indirection. The heuristic should give us at most
    // 16 runs with equal elements, for that range, insertion sort is cheap on binary size and
    // relatively fast.
    insertion_sort_shift_left(runs, 1, &mut |a, b| {
        // SAFETY: The caller must ensure that the runs are valid within `v`.
        unsafe {
            let a_val = v.get_unchecked(a.start);
            let b_val = v.get_unchecked(b.start);

            is_less(a_val, b_val)
        }
    });

    // Now that the runs have the order as they should be when merged, and all comparisons could
    // have been observed. We can 'swap' them in the input. This is achieved efficiently by first
    // copying out the total run area and. Then copying over the relevant areas. None of the
    // following operation is allowed to panic.
    let arr_ptr = v.as_mut_ptr();

    // SAFETY: TODO
    unsafe {
        let mut dest_ptr = arr_ptr.add(left_most_run.start);

        // First save the full region that will be overwritten.
        ptr::copy_nonoverlapping(dest_ptr, buf_ptr, total_run_len);

        for run in runs {
            ptr::copy_nonoverlapping(
                buf_ptr.add(run.start - left_most_run.start),
                dest_ptr,
                run.len,
            );
            dest_ptr = dest_ptr.add(run.len);
        }
    }
}

// --- Branchless sorting (less branches not zero) ---

/// Swap two values in array pointed to by a_ptr and b_ptr if b is less than a.
#[inline(always)]
unsafe fn branchless_swap<T>(a_ptr: *mut T, b_ptr: *mut T, should_swap: bool) {
    // This is a branchless version of swap if.
    // The equivalent code with a branch would be:
    //
    // if should_swap {
    //     ptr::swap_nonoverlapping(a_ptr, b_ptr, 1);
    // }

    // Give ourselves some scratch space to work with.
    // We do not have to worry about drops: `MaybeUninit` does nothing when dropped.
    let mut tmp = mem::MaybeUninit::<T>::uninit();

    // The goal is to generate cmov instructions here.
    let a_swap_ptr = if should_swap { b_ptr } else { a_ptr };
    let b_swap_ptr = if should_swap { a_ptr } else { b_ptr };

    // SAFETY: the caller must guarantee that `a_ptr` and `b_ptr` are valid for writes
    // and properly aligned, and part of the same allocation, and do not alias.
    unsafe {
        ptr::copy_nonoverlapping(b_swap_ptr, tmp.as_mut_ptr(), 1);
        ptr::copy(a_swap_ptr, a_ptr, 1);
        ptr::copy_nonoverlapping(tmp.as_ptr(), b_ptr, 1);
    }
}

/// Swap two values in array pointed to by a_ptr and b_ptr if b is less than a.
#[inline(always)]
unsafe fn swap_if_less<T, F>(arr_ptr: *mut T, a: usize, b: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: the caller must guarantee that `a` and `b` each added to `arr_ptr` yield valid
    // pointers into `arr_ptr`. and properly aligned, and part of the same allocation, and do not
    // alias. `a` and `b` must be different numbers.
    unsafe {
        debug_assert!(a != b);

        let a_ptr = arr_ptr.add(a);
        let b_ptr = arr_ptr.add(b);

        // PANIC SAFETY: if is_less panics, no scratch memory was created and the slice should still be
        // in a well defined state, without duplicates.

        // Important to only swap if it is more and not if it is equal. is_less should return false for
        // equal, so we don't swap.
        let should_swap = is_less(&*b_ptr, &*a_ptr);

        branchless_swap(a_ptr, b_ptr, should_swap);
    }
}

/// Comparing and swapping anything but adjacent elements will yield a non stable sort.
/// So this must be fundamental building block for stable sorting networks.
#[inline(always)]
unsafe fn swap_next_if_less<T, F>(arr_ptr: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: the caller must guarantee that `arr_ptr` and `arr_ptr.add(1)` yield valid
    // pointers that are properly aligned, and part of the same allocation.
    unsafe {
        swap_if_less(arr_ptr, 0, 1, is_less);
    }
}

/// Sort 8 elements
///
/// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
/// performance impact.
#[inline(never)]
fn sort8_stable<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 8.
    assert!(v.len() == 8);

    let arr_ptr = v.as_mut_ptr();

    // SAFETY: We checked the len.
    unsafe {
        // Transposition sorting-network, by only comparing and swapping adjacent wires we have a stable
        // sorting-network. Sorting-networks are great at leveraging Instruction-Level-Parallelism
        // (ILP), they expose multiple comparisons in straight-line code with builtin data-dependency
        // parallelism and ordering per layer. This has to do 28 comparisons in contrast to the 19
        // comparisons done by an optimal size 8 unstable sorting-network.
        swap_next_if_less(arr_ptr.add(0), is_less);
        swap_next_if_less(arr_ptr.add(2), is_less);
        swap_next_if_less(arr_ptr.add(4), is_less);
        swap_next_if_less(arr_ptr.add(6), is_less);

        swap_next_if_less(arr_ptr.add(1), is_less);
        swap_next_if_less(arr_ptr.add(3), is_less);
        swap_next_if_less(arr_ptr.add(5), is_less);

        swap_next_if_less(arr_ptr.add(0), is_less);
        swap_next_if_less(arr_ptr.add(2), is_less);
        swap_next_if_less(arr_ptr.add(4), is_less);
        swap_next_if_less(arr_ptr.add(6), is_less);

        swap_next_if_less(arr_ptr.add(1), is_less);
        swap_next_if_less(arr_ptr.add(3), is_less);
        swap_next_if_less(arr_ptr.add(5), is_less);

        swap_next_if_less(arr_ptr.add(0), is_less);
        swap_next_if_less(arr_ptr.add(2), is_less);
        swap_next_if_less(arr_ptr.add(4), is_less);
        swap_next_if_less(arr_ptr.add(6), is_less);

        swap_next_if_less(arr_ptr.add(1), is_less);
        swap_next_if_less(arr_ptr.add(3), is_less);
        swap_next_if_less(arr_ptr.add(5), is_less);

        swap_next_if_less(arr_ptr.add(0), is_less);
        swap_next_if_less(arr_ptr.add(2), is_less);
        swap_next_if_less(arr_ptr.add(4), is_less);
        swap_next_if_less(arr_ptr.add(6), is_less);

        swap_next_if_less(arr_ptr.add(1), is_less);
        swap_next_if_less(arr_ptr.add(3), is_less);
        swap_next_if_less(arr_ptr.add(5), is_less);
    }
}

/// Sort 16 elements
///
/// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
/// performance impact.
#[inline(never)]
fn sort16_stable<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    assert!(v.len() == 16 && !has_direct_iterior_mutability::<T>());

    sort8_stable(&mut v[0..8], is_less);
    sort8_stable(&mut v[8..16], is_less);

    let mut swap = mem::MaybeUninit::<[T; 16]>::uninit();
    let swap_ptr = swap.as_mut_ptr() as *mut T;

    // SAFETY: We checked that T is Copy and thus observation safe. Should is_less panic v
    // was not modified in bi_directional_merge_even and retains it's original input. swap
    // and v must not alias and swap has v.len() space.
    unsafe {
        bi_directional_merge_even(v, swap_ptr, is_less);
        ptr::copy_nonoverlapping(swap_ptr, v.as_mut_ptr(), 16);
    }
}

fn sort32_stable<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    assert!(v.len() == 32 && !has_direct_iterior_mutability::<T>());

    let mut swap = mem::MaybeUninit::<[T; 32]>::uninit();
    let swap_ptr = swap.as_mut_ptr() as *mut T;

    // SAFETY: We checked the len for sort8 and that T is Copy and thus observation safe.
    // Should is_less panic v was not modified in bi_directional_merge_even and retains it's original input.
    // swap and v must not alias and swap has v.len() space.
    unsafe {
        sort8_stable(&mut v[0..8], is_less);
        sort8_stable(&mut v[8..16], is_less);
        bi_directional_merge_even(&v[0..16], swap_ptr, is_less);

        sort8_stable(&mut v[16..24], is_less);
        sort8_stable(&mut v[24..32], is_less);
        bi_directional_merge_even(&v[16..32], swap_ptr.add(16), is_less);

        let arr_ptr = v.as_mut_ptr();
        ptr::copy_nonoverlapping(swap_ptr, arr_ptr, 32);

        // It's slightly faster to merge directly into v and copy over the 'safe' elements of swap
        // into v only if there was a panic. This technique is also known as ping-pong merge.
        let drop_guard = DropGuard {
            src: swap_ptr,
            dest: arr_ptr,
        };
        bi_directional_merge_even(&*ptr::slice_from_raw_parts(swap_ptr, 32), arr_ptr, is_less);
        mem::forget(drop_guard);
    }

    struct DropGuard<T> {
        src: *const T,
        dest: *mut T,
    }

    impl<T> Drop for DropGuard<T> {
        fn drop(&mut self) {
            // SAFETY: `T` is not a zero-sized type, src must hold the original 32 elements of v in
            // any order. And dest must be valid to write 32 elements.
            unsafe {
                ptr::copy_nonoverlapping(self.src, self.dest, 32);
            }
        }
    }
}

#[inline(never)]
fn panic_on_ord_violation() -> ! {
    panic!("Ord violation");
}
