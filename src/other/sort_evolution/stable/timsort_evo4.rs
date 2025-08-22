/// Difference to evo3, sort network to speedup provide_sorted_batch.
use std::cmp;
use std::cmp::Ordering;
use std::mem::{self, SizedTypeProperties};
use std::ptr;

sort_impl!("timsort_evo4_stable");

#[inline]
pub fn sort<T>(v: &mut [T])
where
    T: Ord,
{
    stable_sort(v, |a, b| a.cmp(b));
}

#[inline]
pub fn sort_by<T, F>(v: &mut [T], compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    stable_sort(v, compare);
}

////////////////////////////////////////////////////////////////////////////////
// Sorting
////////////////////////////////////////////////////////////////////////////////

#[inline]
fn stable_sort<T, F>(v: &mut [T], mut compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    if T::IS_ZST {
        // Sorting has no meaningful behavior on zero-sized types. Do nothing.
        return;
    }

    merge_sort(v, &mut |a, b| compare(a, b) == Ordering::Less);
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
pub fn merge_sort<T, CmpF>(v: &mut [T], is_less: &mut CmpF)
where
    CmpF: FnMut(&T, &T) -> bool,
{
    // The caller should have already checked that.
    debug_assert!(!T::IS_ZST);

    let len = v.len();

    if len < 2 {
        // These inputs are always sorted.
        return;
    }

    // I'd argue most system are not as memory constrained that the double
    let mut buf = Vec::with_capacity(len); // I'm skipping the fallback len / 2 part here.
    let buf_ptr = buf.as_mut_ptr();

    let mut runs = Vec::new();

    let mut end = 0;
    let mut start = 0;

    // Scan forward. Memory pre-fetching prefers forward scanning vs backwards scanning, and the
    // code-gen is usually better. For the most sensitive types such as integers, these are merged
    // bidirectionally at once. So there is no benefit in scanning backwards.
    while end < len {
        let (streak_end, was_reversed) = find_streak(&v[start..], is_less);
        end += streak_end;
        if was_reversed {
            v[start..end].reverse();
        }

        // Insert some more elements into the run if it's too short. Insertion sort is faster than
        // merge sort on short sequences, so this significantly improves performance.
        end = provide_sorted_batch(v, start, end, is_less);

        // Push this run onto the stack.
        runs.push(TimSortRun {
            start,
            len: end - start,
        });
        start = end;

        // Merge some pairs of adjacent runs to satisfy the invariants.
        while let Some(r) = collapse(runs.as_slice(), len) {
            let left = runs[r];
            let right = runs[r + 1];
            let merge_slice = &mut v[left.start..right.start + right.len];
            unsafe {
                if qualifies_for_parity_merge::<T>() {
                    parity_merge_plus(merge_slice, left.len, buf_ptr, is_less);
                    ptr::copy_nonoverlapping(buf_ptr, merge_slice.as_mut_ptr(), merge_slice.len());
                } else {
                    merge(merge_slice, left.len, buf_ptr, is_less);
                }
            }
            runs[r + 1] = TimSortRun {
                start: left.start,
                len: left.len + right.len,
            };
            runs.remove(r);
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
    #[inline]
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
}

/// Internal type used by merge_sort.
#[derive(Clone, Copy, Debug)]
pub struct TimSortRun {
    len: usize,
    start: usize,
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
                let len = self.end.offset_from_unsigned(self.start);
                ptr::copy_nonoverlapping(self.start, self.dest, len);
            }
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

    if qualifies_for_parity_merge::<T>()
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

// I would like to make this a const fn.
#[inline]
fn qualifies_for_parity_merge<T>() -> bool {
    // This checks two things:
    //
    // - Type size: Is it ok to create 40 of them on the stack.
    //
    // - Can the type have interior mutability, this is checked by testing if T is Copy.
    //   If the type can have interior mutability it may alter itself during comparison in a way
    //   that must be observed after the sort operation concludes.
    //   Otherwise a type like Mutex<Option<Box<str>>> could lead to double free.

    let is_small = mem::size_of::<T>() <= mem::size_of::<[usize; 2]>();
    let is_copy = T::is_copy();

    return is_small && is_copy;
}

#[inline]
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

#[inline]
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
        panic!("Ord violation");
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

    assert!(mid > 0 && mid < len);

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
            panic!("Ord violation");
        }

        if !left_ptr_done {
            // Be vigilant and check everything that could go wrong.
            // t_ptr_left must be within the left side and larger or equal to ptr_left.
            if !(t_ptr_data >= ptr_data && t_ptr_left < mid_ptr && t_ptr_left >= ptr_left) {
                panic!("Ord violation");
            }

            let buf_rest_len = t_ptr_data.offset_from_unsigned(ptr_data) + 1;
            let copy_len = t_ptr_left.offset_from_unsigned(ptr_left) + 1;
            assert!(copy_len == buf_rest_len);
            ptr::copy_nonoverlapping(ptr_left, ptr_data, copy_len);
            ptr_left = ptr_left.add(copy_len);
        } else if !right_ptr_done {
            // t_ptr_right must be within the right side and larger or equal to ptr_right.
            if !(t_ptr_data >= ptr_data && t_ptr_right < end_ptr && t_ptr_right >= ptr_right) {
                panic!("Ord violation");
            }

            let buf_rest_len = t_ptr_data.offset_from_unsigned(ptr_data) + 1;
            let copy_len = t_ptr_right.offset_from_unsigned(ptr_right) + 1;
            assert!(copy_len == buf_rest_len);
            ptr::copy_nonoverlapping(ptr_right, ptr_data, copy_len);
            ptr_right = ptr_right.add(copy_len);
        }
    }

    let left_diff = calc_ptr_diff(ptr_left, t_ptr_left);
    let right_diff = calc_ptr_diff(ptr_right, t_ptr_right);

    if !(left_diff == mem::size_of::<T>() && right_diff == mem::size_of::<T>()) {
        panic!("Ord violation");
    }
}

// --- Branchless sorting (less branches not zero) ---

/// Swap two values in array pointed to by a_ptr and b_ptr if b is less than a.
#[inline]
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
#[inline]
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
#[inline]
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
    assert!(v.len() == 32 && T::is_copy());

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
