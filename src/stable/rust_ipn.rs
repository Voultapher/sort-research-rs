#![allow(unused)]

//! Instruction-Parallel-Network Stable Sort by Lukas Bergdoll

use std::alloc;
use std::cmp;
use std::cmp::Ordering;
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
#[cfg(not(no_global_oom_handling))]
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

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
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
    RunAllocF: Fn(usize) -> *mut TimSortRun,
    RunDeallocF: Fn(*mut TimSortRun, usize),
{
    // The caller should have already checked that.
    debug_assert!(!T::IS_ZST);

    let len = v.len();

    if len < 2 {
        // These inputs are always sorted.
        return;
    }

    if sort_small_stable(v, &mut |a, b| compare(a, b) == Ordering::Less) {
        return;
    }

    let buf_len_wish = if has_no_direct_iterior_mutability::<T>() {
        len
    } else {
        len / 2
    };

    let buf_len_fallback_min = len / 2;
    let buf = BufGuard::new(
        buf_len_wish,
        buf_len_fallback_min,
        elem_alloc_fn,
        elem_dealloc_fn,
    );
    let buf_ptr = buf.buf_ptr.as_ptr();
    let buf_len = buf.capacity;

    // Experiments with stack allocation for small inputs showed worse performance.
    // May depend on the platform.
    merge_sort_impl(v, compare, buf_ptr, buf_len, run_alloc_fn, run_dealloc_fn);

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

pub fn merge_sort_impl<T, CmpF, RunAllocF, RunDeallocF>(
    v: &mut [T],
    compare: &mut CmpF,
    buf_ptr: *mut T, // TODO mem::MaybeUninit<&mut [T]>,
    buf_len: usize,
    run_alloc_fn: RunAllocF,
    run_dealloc_fn: RunDeallocF,
) where
    CmpF: FnMut(&T, &T) -> Ordering,
    RunAllocF: Fn(usize) -> *mut TimSortRun,
    RunDeallocF: Fn(*mut TimSortRun, usize),
{
    // The caller should have already checked that.
    debug_assert!(!T::IS_ZST);

    let len = v.len();

    const MIN_REPROBE_DISTANCE: usize = 256;
    let max_probe_begin_len: usize = cmp::max(len / MIN_REPROBE_DISTANCE, 20);
    // Limit the possibility of doing consecutive ineffective partitions.
    let min_good_partiton_len: usize = ((len as f64).log2() * 2.0).round() as usize;

    let mut runs = RunVec::new(&run_alloc_fn, &run_dealloc_fn);

    let mut start = 0;
    let mut end = 0;
    let mut next_probe_spot = 0;
    let mut all_equal = false;

    // Scan forward. Memory pre-fetching prefers forward scanning vs backwards scanning, and the
    // code-gen is usually better. For the most sensitive types such as integers, these are merged
    // bidirectionally at once. So there is no benefit in scanning backwards.
    while end < len {
        let probe_for_common = start >= next_probe_spot && (len - start) <= buf_len;

        // SAFETY: We checked that buf_ptr can hold `v[start..]` if probe_for_common is true.
        (end, all_equal) =
            unsafe { natural_sort(&mut v[start..], buf_ptr, probe_for_common, compare) };

        // Avoid re-probing the same area again and again if probing failed or was of low
        // quality.
        next_probe_spot = if !probe_for_common || (all_equal && end >= min_good_partiton_len) {
            next_probe_spot
        } else {
            start + MIN_REPROBE_DISTANCE
        };

        end += start;

        // Insert some more elements into the run if it's too short. Insertion sort is faster than
        // merge sort on short sequences, so this significantly improves performance.
        let new_end =
            provide_sorted_batch(v, start, end, &mut |a, b| compare(a, b) == Ordering::Less);

        // The all_equal assertion only holds if provide_sorted_batch did *not* extend the sorted
        // batch.
        all_equal = all_equal && new_end == end;
        end = new_end;

        // Push this run onto the stack.
        runs.push(TimSortRun {
            start,
            len: end - start,
            all_equal,
        });
        start = end;

        // type DebugT = (i32, i32);
        // let mut check_vec = Vec::new();

        // Merge some pairs of adjacent runs to satisfy the invariants.
        while let Some(r) = collapse(runs.as_slice(), len) {
            let left = runs[r];
            let right = runs[r + 1];
            let merge_slice = &mut v[left.start..right.start + right.len];
            unsafe {
                if (left.all_equal || right.all_equal) && merge_slice.len() <= buf_len {
                    // check_vec = mem::transmute::<&[T], &[DebugT]>(merge_slice).to_vec();

                    merge_run_with_equal(merge_slice, buf_ptr, &left, &right, compare);
                } else if has_no_direct_iterior_mutability::<T>() && merge_slice.len() <= buf_len {
                    parity_merge_plus(merge_slice, left.len, buf_ptr, &mut |a, b| {
                        compare(a, b) == Ordering::Less
                    });
                    ptr::copy_nonoverlapping(buf_ptr, merge_slice.as_mut_ptr(), merge_slice.len());
                } else {
                    merge(merge_slice, left.len, buf_ptr, &mut |a, b| {
                        compare(a, b) == Ordering::Less
                    });
                }
            }
            runs[r + 1] = TimSortRun {
                start: left.start,
                len: left.len + right.len,
                all_equal: false,
            };
            runs.remove(r);

            // if (left.all_equal || right.all_equal) {
            //     check_vec.sort();
            //     let x = unsafe { mem::transmute::<&[T], &[DebugT]>(merge_slice) };
            //     assert_eq!(x, check_vec);
            // }
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

    struct RunVec<RunAllocF, RunDeallocF>
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

    impl<RunAllocF, RunDeallocF> RunVec<RunAllocF, RunDeallocF>
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

        fn len(&self) -> usize {
            self.len
        }
    }

    impl<RunAllocF, RunDeallocF> core::ops::Index<usize> for RunVec<RunAllocF, RunDeallocF>
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

    impl<RunAllocF, RunDeallocF> core::ops::IndexMut<usize> for RunVec<RunAllocF, RunDeallocF>
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

    impl<RunAllocF, RunDeallocF> Drop for RunVec<RunAllocF, RunDeallocF>
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
}

/// Internal type used by merge_sort.
#[derive(Clone, Copy, Debug)]
pub struct TimSortRun {
    len: usize,
    start: usize,
    all_equal: bool,
}

/// Analyzes region at the start of `v` and will leverage natural patterns contained in the input.
/// Returns offset until where `v[..end]` will be sorted after the call, and bool denoting wether
/// that area contains only equal elements.
///
/// SAFETY: Caller must ensure if probe_for_common is set to true that `buf` is valid for `v.len()`
/// writes.
unsafe fn natural_sort<T, F>(
    v: &mut [T],
    buf: *mut T,
    probe_for_common: bool,
    compare: &mut F,
) -> (usize, bool)
where
    F: FnMut(&T, &T) -> Ordering,
{
    // Starting at this streak size the one additional comparison is not too expensive.
    const MIN_ASCENDING_ALL_EQUAL_CHECK: usize = 12;

    let len = v.len();
    if len < 2 {
        return (len, false);
    }

    let probe_result = probe_region(v, probe_for_common, compare);
    match probe_result {
        ProbeResult::Ascending(end) => {
            let all_equal = if end >= MIN_ASCENDING_ALL_EQUAL_CHECK {
                // compare(&v[0], &v[end - 1]) == Ordering::Equal
                // TODO enable this but it makes merge_run_with_equal more complicated.
                false
            } else {
                false
            };
            (end, all_equal)
        }
        ProbeResult::Descending(end) => {
            // These can't be all equal.
            v[..end].reverse();
            (end, false)
        }
        ProbeResult::CommonValue(idx) => {
            // SAFETY: Caller must ensure if probe_for_common is set to true that `buf` is valid for
            // `v.len()` writes.
            let end = unsafe {
                partition_equal_stable(v, idx, buf, &mut |a, b| compare(a, b) == Ordering::Equal)
            };

            (end, true)
        }
    }
}

#[derive(Copy, Clone)]
enum ProbeResult {
    Ascending(usize),   // Offset until where it is sorted.
    Descending(usize),  // Offset until where it is sorted descending.
    CommonValue(usize), // Offset of element that is common.
}

/// Analyzes region at the start of `v` at tries to find these types of patterns:
/// A) Fully ascending
/// B) Full descending
/// C) Common value
fn probe_region<T, F>(v: &[T], probe_for_common: bool, compare: &mut F) -> ProbeResult
where
    F: FnMut(&T, &T) -> Ordering,
{
    // Probe for common value with priority over streak analysis.
    if probe_for_common {
        let probe_common_result =
            probe_for_common_val(v, &mut |a, b| compare(a, b) == Ordering::Equal);

        if let Some(idx) = probe_common_result {
            return ProbeResult::CommonValue(idx);
        }
    }

    let (streak_end, was_reversed) = find_streak(v, &mut |a, b| compare(a, b) == Ordering::Less);
    return if was_reversed {
        ProbeResult::Descending(streak_end)
    } else {
        ProbeResult::Ascending(streak_end)
    };
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

/// Partition `v` into elements that are equal to `v[pivot_pos]` followed by elements not equal to
/// `v[pivot_pos]`. Relative position of `v[pivot_pos]` is maintained.
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

    let arr_ptr = v.as_ptr();

    // SAFETY: The caller must ensure `buf` is valid for `v.len()` writes.
    // See specific comments below.
    unsafe {
        let pivot_val = mem::ManuallyDrop::new(ptr::read(&v[pivot_pos]));
        // It's crucial that pivot_hole will be copied back to the input if any comparison in the
        // loop panics. Because it could have changed due to interior mutability.
        let pivot_hole = InsertionHole {
            src: &*pivot_val,
            dest: v.as_mut_ptr().add(pivot_pos),
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

            swap_ptr_l = swap_ptr_l.add(!is_eq as usize);
            swap_ptr_r = swap_ptr_r.sub(is_eq as usize);
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
        ptr::copy_nonoverlapping(buf.add(l_count), arr_ptr, r_count);
        v[..r_count].reverse();

        let arr_ptr = v.as_mut_ptr();
        ptr::copy_nonoverlapping(buf, arr_ptr.add(r_count), l_count);

        r_count
    }
}

/// Merges left and right run, assuming at least one of them contains only equal elements. SAFETY:
/// buf must be able to hold at least v.len(), and both sides must hold at least 1 element.
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
#[inline(never)]
unsafe fn merge_run_with_equal<T, F>(
    v: &mut [T],
    buf: *mut T,
    left: &TimSortRun,
    right: &TimSortRun,
    compare: &mut F,
) where
    F: FnMut(&T, &T) -> Ordering,
{
    assert!(left.len >= 1 && right.len >= 1 && v.len() == left.len + right.len);

    let comp_l0_r0 = compare(&v[0], &v[left.len]);
    let comp_l0_r_end = compare(&v[0], &v[v.len() - 1]);
    let arr_ptr = v.as_mut_ptr();

    // All runs that are marked all_equal imply they have not been merged before, as doing so loses
    // the label. We know partition_equal_stable is guaranteed to find all elements that are equal.
    // From this follows, if both runs are all_equal they cannot have the same elements, the
    // previous partition_equal_stable would have already found them all. However it's possible that
    // a previous run contains elements equal to the run that is all_equal, because they weren't
    // found by probe_for_common_val. These must have been found before the call to
    // partition_equal_stable. As consequence in both cases we have to find the last element that is
    // equal or its according spot and insert the all_equal run there.
    //
    // However there is another way to create a run that is all equal, and that's ascending with
    // first and last element being the same. This does not guarantee that the run will hold all the
    // remaining elements that are equal. Yet we know that both types of runs are contiguous and we
    // know their start position. By comparing their start positions we can know in what order they
    // must appear. They cannot interleave, one must come before the other.

    // type DebugT = (i32, i32);
    // let v_as_x = mem::transmute::<&[T], &[DebugT]>(v);
    // println!(
    //     "\n\nLEFT:\n{:?}\nRight:\n{:?}",
    //     &v_as_x[..left.len],
    //     &v_as_x[left.len..]
    // );
    // dbg!(left, right);

    if (left.all_equal && right.all_equal)
        || (left.all_equal && comp_l0_r_end == Ordering::Equal && left.start > right.start)
    {
        debug_assert!(compare(&v[0], &v[left.len]) != Ordering::Equal);

        let run_comes_after = comp_l0_r0 == Ordering::Greater
            || (comp_l0_r0 == Ordering::Equal && left.start > right.start);

        if run_comes_after {
            // SAFETY: TODO
            unsafe {
                // Swap left and right side.
                ptr::copy_nonoverlapping(arr_ptr, buf, left.len);
                ptr::copy(arr_ptr.add(left.len), arr_ptr, right.len);
                ptr::copy_nonoverlapping(buf, arr_ptr.add(right.len), left.len);
            }
        }
        return;
    }

    if left.all_equal {
        if comp_l0_r0 != Ordering::Less {
            let l_elem = &v[0];

            let insert_pos = match v[left.len..].binary_search_by(|elem| compare(elem, l_elem)) {
                Ok(val) => {
                    let run_comes_after = left.start > right.start;
                    assert!(run_comes_after);

                    let offset = left.len + val;

                    let rel_pos = v[offset..]
                        .iter()
                        .position(|elem| compare(elem, l_elem) != Ordering::Equal)
                        .unwrap(); // TODO explain unwrap Ord violation.

                    if rel_pos > (v.len() - offset) {
                        panic_on_ord_violation();
                    }

                    offset + rel_pos

                    // // This is necessary to make this stable.
                    // let run_comes_after = left.start > right.start;
                    // if run_comes_after {
                    //     let rel_pos = v[offset..]
                    //         .iter()
                    //         .position(|elem| compare(elem, l_elem) != Ordering::Equal)
                    //         .unwrap(); // TODO explain unwrap Ord violation.

                    //     offset + rel_pos
                    // } else {
                    //     let rel_pos = v[..offset]
                    //         .iter()
                    //         .rev()
                    //         .position(|elem| compare(elem, l_elem) != Ordering::Equal)
                    //         .unwrap(); // TODO explain unwrap Ord violation.

                    //     offset - (rel_pos + 1)
                    // }
                }
                Err(pos) => pos,
            };

            // SAFETY: TODO
            unsafe {
                // Safe left side in buf.
                ptr::copy_nonoverlapping(arr_ptr, buf, left.len);
                // Copy everything from right side up to insert_pos over left side.
                // This can overlap.
                ptr::copy(arr_ptr.add(left.len), arr_ptr, insert_pos);
                // Now copy the left side into the hole.
                ptr::copy_nonoverlapping(buf, arr_ptr.add(insert_pos), left.len);
            }
        }
        return;
    }

    debug_assert!(right.all_equal);

    let is_less_r0_l_end = compare(&v[left.len], &v[left.len - 1]) == Ordering::Less;

    // If the the right elements are all more or equal than the last element of the assumed sorted
    // left side, they are already in the correct spot.
    if is_less_r0_l_end {
        let r_elem = &v[left.len];

        let insert_pos = match v[..left.len].binary_search_by(|elem| compare(elem, r_elem)) {
            Ok(val) => {
                let run_comes_after = right.start > left.start;
                assert!(run_comes_after); // TODO why is this always true?

                let rel_pos = v[val..]
                    .iter()
                    .position(|elem| compare(elem, r_elem) != Ordering::Equal)
                    .unwrap(); // TODO explain unwrap Ord violation.

                val + rel_pos
            }
            Err(pos) => pos,
        };

        // SAFETY: TODO
        unsafe {
            if insert_pos > left.len {
                panic_on_ord_violation();
            }

            let overwrite_left_count = left.len - insert_pos;
            // Safe all elements from left side that would be overwritten.
            ptr::copy_nonoverlapping(arr_ptr.add(insert_pos), buf, overwrite_left_count);
            // Copy everything from right side into the hole, this could overlap.
            ptr::copy(arr_ptr.add(left.len), arr_ptr.add(insert_pos), right.len);
            // Now copy the saved elements back to the end.
            ptr::copy_nonoverlapping(
                buf,
                arr_ptr.add(v.len() - overwrite_left_count),
                overwrite_left_count,
            );
        }
    }
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn sort_small_stable<T, F>(v: &mut [T], is_less: &mut F) -> bool
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    // Slices of up to this length get sorted using optimized sorting for small slices.
    const fn max_len_small_sort_stable<T>() -> usize {
        if is_cheap_to_move::<T>() {
            32
        } else {
            20
        }
    }

    if len > max_len_small_sort_stable::<T>() {
        return false;
    }

    let (end, was_reversed) = find_streak(v, is_less);
    if was_reversed {
        v[..end].reverse();
    }

    if end == len {
        return true;
    }

    if is_cheap_to_move::<T>() {
        // Testing showed that even though this incurs more comparisons, up to size 32 (4 * 8),
        // avoiding the allocation and sticking with simple code is worth it. Going further eg. 40
        // is still worth it for u64 or even types with more expensive comparisons, but risks
        // incurring just too many comparisons than doing the regular TimSort.
        if len < 8 {
            insertion_sort_shift_left(v, end, is_less);
            insertion_sort_shift_left(v, 1, is_less);
            return true;
        } else if len < 16 {
            sort8_stable(&mut v[0..8], is_less);
            insertion_sort_shift_left(v, 8, is_less);
            return true;
        }

        // This should optimize to a shift right https://godbolt.org/z/vYGsznPPW.
        let even_len = len - (len % 2 != 0) as usize;
        let len_div_2 = even_len / 2;

        sort8_stable(&mut v[0..8], is_less);
        sort8_stable(&mut v[len_div_2..(len_div_2 + 8)], is_less);

        insertion_sort_shift_left(&mut v[0..len_div_2], 8, is_less);
        insertion_sort_shift_left(&mut v[len_div_2..], 8, is_less);

        let mut swap = mem::MaybeUninit::<[T; max_len_small_sort_stable::<i32>()]>::uninit();
        let swap_ptr = swap.as_mut_ptr() as *mut T;

        if has_no_direct_iterior_mutability::<T>() {
            // SAFETY: We checked that T is Copy and thus observation safe.
            // Should is_less panic v was not modified in parity_merge and retains it's original input.
            // swap and v must not alias and swap has v.len() space.
            unsafe {
                parity_merge(&mut v[..even_len], swap_ptr, is_less);
                ptr::copy_nonoverlapping(swap_ptr, v.as_mut_ptr(), even_len);
            }
        } else {
            // SAFETY: swap_ptr can hold v.len() elements and both sides are at least of len 1.
            unsafe {
                merge(&mut v[..even_len], len_div_2, swap_ptr, is_less);
            }
        }

        if len != even_len {
            // SAFETY: We know len >= 2.
            unsafe {
                insert_tail(v, is_less);
            }
        }

        return true;
    } else {
        insertion_sort_shift_left(v, end, is_less);
        return true;
    }

    true
}

/// Takes a range as denoted by start and end, that is already sorted and extends it to the right if
/// necessary with sorts optimized for smaller ranges such as insertion sort.
#[cfg(not(no_global_oom_handling))]
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn provide_sorted_batch<T, F>(v: &mut [T], start: usize, mut end: usize, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    assert!(end >= start && end <= len);

    // This value is a balance between least comparisons and best performance, as
    // influenced by for example cache locality.
    const MIN_INSERTION_RUN: usize = 10;
    const MAX_IGNORE_PRE_SORTED: usize = 6;

    // Insert some more elements into the run if it's too short. Insertion sort is faster than
    // merge sort on short sequences, so this significantly improves performance.
    let start_end_diff = end - start;

    const FAST_SORT_SIZE: usize = 32;

    if is_cheap_to_move::<T>()
        && has_no_direct_iterior_mutability::<T>()
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
    } else if start_end_diff < MIN_INSERTION_RUN && end < len {
        // v[start_found..end] are elements that are already sorted in the input. We want to extend
        // the sorted region to the left, so we push up MIN_INSERTION_RUN - 1 to the right. Which is
        // more efficient that trying to push those already sorted elements to the left.
        end = cmp::min(start + MIN_INSERTION_RUN, len);
        let presorted_start = cmp::max(start_end_diff, 1);

        insertion_sort_shift_left(&mut v[start..end], presorted_start, is_less);
    }

    end
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
///
/// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
/// performance impact. Even improving performance in some cases.
#[inline(never)]
fn insertion_sort_shift_left<T, F>(v: &mut [T], offset: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    // This would be a logic bug.
    // Using assert here improves performance.
    assert!(offset != 0 && offset <= len);

    // Shift each element of the unsorted region v[i..] as far left as is needed to make v sorted.
    for i in offset..len {
        // SAFETY: we tested that len >= 2.
        unsafe {
            // Maybe use insert_head here and avoid additional code.
            insert_tail(&mut v[..=i], is_less);
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
///
/// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
/// performance impact.
#[inline(never)]
#[cfg(not(no_global_oom_handling))]
unsafe fn merge<T, F>(v: &mut [T], mid: usize, buf: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    assert!(mid > 0 && mid < len);

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
                // let is_l = is_less(&*right, &**left);
                // let copy_ptr = if is_l { right } else { *left };
                // ptr::copy_nonoverlapping(copy_ptr, *out, 1);
                // right = right.wrapping_add(is_l as usize);
                // *left = left.wrapping_add(!is_l as usize);
                // *out = out.add(1);

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
fn has_no_direct_iterior_mutability<T>() -> bool {
    // - Can the type have interior mutability, this is checked by testing if T is Copy.
    //   If the type can have interior mutability it may alter itself during comparison in a way
    //   that must be observed after the sort operation concludes.
    //   Otherwise a type like Mutex<Option<Box<str>>> could lead to double free.
    //   FIXME use proper abstraction
    T::is_copy()
}

#[inline(always)]
pub unsafe fn merge_up<T, F>(
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

    let is_l = !is_less(&*src_right, &*src_left);
    let copy_ptr = if is_l { src_left } else { src_right };
    ptr::copy_nonoverlapping(copy_ptr, dest_ptr, 1);
    src_right = src_right.wrapping_add(!is_l as usize);
    src_left = src_left.wrapping_add(is_l as usize);
    dest_ptr = dest_ptr.add(1);

    (src_left, src_right, dest_ptr)
}

#[inline(always)]
pub unsafe fn merge_down<T, F>(
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

    let is_l = !is_less(&*src_right, &*src_left);
    let copy_ptr = if is_l { src_right } else { src_left };
    ptr::copy_nonoverlapping(copy_ptr, dest_ptr, 1);
    src_right = src_right.wrapping_sub(is_l as usize);
    src_left = src_left.wrapping_sub(!is_l as usize);
    dest_ptr = dest_ptr.sub(1);

    (src_left, src_right, dest_ptr)
}

/// Merge v assuming the len is even and v[..len / 2] and v[len / 2..] are sorted.
///
/// Adapted from crumsort/quadsort.
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
pub unsafe fn parity_merge<T, F>(v: &[T], dest_ptr: *mut T, is_less: &mut F)
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

    assert!(has_no_direct_iterior_mutability::<T>());

    let len = v.len();
    let src_ptr = v.as_ptr();

    let len_div_2 = len / 2;

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

/// Merge v assuming v[..mid] and v[mid..] are already sorted.
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
pub unsafe fn parity_merge_plus<T, F>(v: &[T], mid: usize, dest_ptr: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: the caller must guarantee that `dest_ptr` is valid for v.len() writes.
    // Also `v.as_ptr` and `dest_ptr` must not alias.
    //
    // The caller must guarantee that T cannot modify itself inside is_less.

    let len = v.len();
    let src_ptr = v.as_ptr();

    assert!(mid > 0 && mid < len && has_no_direct_iterior_mutability::<T>());

    // TODO explain why this is fast.

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

    // TODO explain why this is safe even with Ord violations.
    for _ in 0..shorter_side {
        (ptr_left, ptr_right, ptr_data) = merge_up(ptr_left, ptr_right, ptr_data, is_less);
        (t_ptr_left, t_ptr_right, t_ptr_data) =
            merge_down(t_ptr_left, t_ptr_right, t_ptr_data, is_less);
    }

    let calc_ptr_diff = |ptr, base_ptr| (ptr as usize).wrapping_sub(base_ptr as usize);

    if shorter_side != longer_side {
        // TODO explain loop conditions and Ord violation overlap.
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

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn sort32_stable<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    assert!(v.len() == 32 && has_no_direct_iterior_mutability::<T>());

    let mut swap = mem::MaybeUninit::<[T; 32]>::uninit();
    let swap_ptr = swap.as_mut_ptr() as *mut T;

    // SAFETY: We checked the len for sort8 and that T is Copy and thus observation safe.
    // Should is_less panic v was not modified in parity_merge and retains it's original input.
    // swap and v must not alias and swap has v.len() space.
    unsafe {
        sort8_stable(&mut v[0..8], is_less);
        sort8_stable(&mut v[8..16], is_less);
        parity_merge(&v[0..16], swap_ptr, is_less);

        sort8_stable(&mut v[16..24], is_less);
        sort8_stable(&mut v[24..32], is_less);
        parity_merge(&v[16..32], swap_ptr.add(16), is_less);

        let arr_ptr = v.as_mut_ptr();
        ptr::copy_nonoverlapping(swap_ptr, arr_ptr, 32);

        // It's slightly faster to merge directly into v and copy over the 'safe' elements of swap
        // into v only if there was a panic.
        let drop_guard = DropGuard {
            src: swap_ptr,
            dest: arr_ptr,
        };
        parity_merge(&*ptr::slice_from_raw_parts(swap_ptr, 32), arr_ptr, is_less);
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
