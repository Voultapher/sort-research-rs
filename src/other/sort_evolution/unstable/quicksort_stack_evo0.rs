//! Non-recursive quicksort with heapsort fallback.

use std::cmp::Ordering;
use std::mem::{MaybeUninit, SizedTypeProperties};
use std::ptr;

sort_impl!("quicksort_stack_evo0_unstable");

#[inline]
pub fn sort<T>(v: &mut [T])
where
    T: Ord,
{
    unstable_sort(v, |a, b| a.lt(b));
}

#[inline]
pub fn sort_by<T, F>(v: &mut [T], mut compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    unstable_sort(v, |a, b| compare(a, b) == Ordering::Less);
}

////////////////////////////////////////////////////////////////////////////////
// Sorting
////////////////////////////////////////////////////////////////////////////////

#[inline]
#[cfg(not(no_global_oom_handling))]
fn unstable_sort<T, F>(v: &mut [T], mut is_less: F)
where
    F: FnMut(&T, &T) -> bool,
{
    if T::IS_ZST {
        // Sorting has no meaningful behavior on zero-sized types. Do nothing.
        return;
    }

    // Limit the number of imbalanced partitions to `2 * floor(log2(len))`.
    // The binary OR by one is used to eliminate the zero-check in the logarithm.
    let limit = 2 * (v.len() | 1).ilog2();

    quicksort(v, limit, &mut is_less);
}

fn quicksort<T, F>(v_full: &mut [T], mut limit: u32, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    const MAX_DEPTH: u32 = 2 * isize::MAX.ilog2();
    let mut stack_storage = MaybeUninit::<[usize; MAX_DEPTH as usize]>::uninit();
    let stack: *mut usize = stack_storage.as_mut_ptr().cast();
    let mut stack_len = 0;

    // SAFETY: Stack is guaranteed large enough, by virtue of MAX_DEPTH and limit.
    unsafe {
        *stack = 0;
        *stack.add(1) = v_full.len();
        stack_len += 2;
    }

    // unsafe {
    //     println!(
    //         "v_full: {:?}",
    //         &*ptr::slice_from_raw_parts(v_full.as_ptr() as *const i32, v_full.len())
    //     );
    // }

    while stack_len != 0 {
        let v_begin_idx;
        let v_end_idx;
        let stack_current;
        // SAFETY: Stack is guaranteed large enough, by virtue of MAX_DEPTH and limit.
        // And `*stack..*stack.add(1)` is always created as in-bounds range.
        let v = unsafe {
            // println!(
            //     "Current stack: {:?}",
            //     &*ptr::slice_from_raw_parts(stack, stack_len)
            // );

            stack_len -= 2;
            stack_current = stack.add(stack_len);
            v_begin_idx = *stack_current;
            v_end_idx = *stack_current.add(1);

            // assert!(v_end_idx >= v_begin_idx, "{v_begin_idx}..{v_end_idx}");
            v_full.get_unchecked_mut(v_begin_idx..v_end_idx)
        };

        let len = v.len();
        if len < 2 {
            continue;
        }

        // If too many bad pivot choices were made, simply fall back to heapsort in order to
        // guarantee `O(n * log(n))` worst-case.
        if limit == 0 {
            // SAFETY: We checked that `len >= 2`.
            unsafe {
                heapsort(v, is_less);
            }
            continue;
        }

        limit -= 1;

        let (pivot, v_without_pivot) = v.split_at_mut(1);
        let pivot = &pivot[0];

        let lt_count = lomuto_partition_branchless(v_without_pivot, pivot, is_less);

        // Place the pivot between the two partitions.
        v.swap(0, lt_count);

        // SAFETY: Stack is guaranteed large enough, by virtue of MAX_DEPTH and limit.
        unsafe {
            *stack_current = v_begin_idx;
            *stack_current.add(1) = v_begin_idx + lt_count;
            *stack_current.add(2) = v_begin_idx + lt_count + 1;
            *stack_current.add(3) = v_end_idx;
            stack_len += 4;
        }
    }
}

fn lomuto_partition_branchless<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    let mut l = 0;
    for r in 0..len {
        let is_lt = is_less(&v[r], pivot);
        v.swap(l, r);
        l += is_lt as usize;
    }

    l
}

/// Sorts `v` using heapsort, which guarantees *O*(*n* \* log(*n*)) worst-case.
///
/// Never inline this, it sits the main hot-loop in `recurse` and is meant as unlikely algorithmic
/// fallback.
///
/// SAFETY: The caller has to guarantee that `v.len()` >= 2.
#[inline(never)]
unsafe fn heapsort<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    if v.len() < 2 {
        // This helps prove things to the compiler. That we checked earlier.
        // SAFETY: This function is only called if len >= 2.
        unsafe {
            core::hint::unreachable_unchecked();
        }
    }

    let len = v.len();

    // Build the heap in linear time.
    for i in (0..len / 2).rev() {
        sift_down(v, i, is_less);
    }

    // Pop maximal elements from the heap.
    for i in (1..len).rev() {
        v.swap(0, i);
        sift_down(&mut v[..i], 0, is_less);
    }
}

// This binary heap respects the invariant `parent >= child`.
//
// SAFETY: The caller has to guarantee that node < `v.len()`.
#[inline(never)]
unsafe fn sift_down<T, F>(v: &mut [T], mut node: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    if node >= v.len() {
        // This helps prove things to the compiler. That we checked earlier.
        // SAFETY: This function is only called if node < `v.len()`.
        unsafe {
            core::hint::unreachable_unchecked();
        }
    }

    let len = v.len();

    let v_base = v.as_mut_ptr();

    loop {
        // Children of `node`.
        let mut child = 2 * node + 1;
        if child >= len {
            break;
        }

        // SAFETY: The invariants and checks guarantee that both node and child are in-bounds.
        unsafe {
            // Choose the greater child.
            if child + 1 < len {
                // We need a branch to be sure not to out-of-bounds index,
                // but it's highly predictable.  The comparison, however,
                // is better done branchless, especially for primitives.
                child += is_less(&*v_base.add(child), &*v_base.add(child + 1)) as usize;
            }

            // Stop if the invariant holds at `node`.
            if !is_less(&*v_base.add(node), &*v_base.add(child)) {
                break;
            }

            // Swap `node` with the greater child, move one step down, and continue sifting.
            // Same as v.swap_unchecked(node, child); which is unstable.
            ptr::swap(v_base.add(node), v_base.add(child))
        }

        node = child;
    }
}
