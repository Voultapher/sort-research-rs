use core::intrinsics;
use core::mem::{self, ManuallyDrop};
use core::ptr;

use crate::smallsort::SmallSortImpl;
use crate::{GapGuard, IsTrue};

/// Sorts `v` recursively.
///
/// If the slice had a predecessor in the original array, it is specified as `ancestor_pivot`.
///
/// `limit` is the number of allowed imbalanced partitions before switching to `heapsort`. If zero,
/// this function will immediately switch to heapsort.
pub(crate) fn quicksort<'a, T, F>(
    mut v: &'a mut [T],
    is_less: &mut F,
    mut ancestor_pivot: Option<&'a T>,
    mut limit: u32,
) where
    F: FnMut(&T, &T) -> bool,
{
    loop {
        // println!("len: {}", v.len());

        if v.len() <= T::SMALL_SORT_THRESHOLD {
            T::small_sort(v, is_less);
            return;
        }

        // If too many bad pivot choices were made, simply fall back to heapsort in order to
        // guarantee `O(n * log(n))` worst-case.
        if limit == 0 {
            // SAFETY: We assume the `small_sort` threshold is at least 1.
            unsafe {
                crate::heapsort::heapsort(v, is_less);
            }
            return;
        }

        limit -= 1;

        // Choose a pivot and try guessing whether the slice is already sorted.
        let pivot_pos = crate::pivot::choose_pivot(v, is_less);

        // If the chosen pivot is equal to the predecessor, then it's the smallest element in the
        // slice. Partition the slice into elements equal to and elements greater than the pivot.
        // This case is usually hit when the slice contains many duplicate elements.
        if let Some(p) = ancestor_pivot {
            // SAFETY: We assume choose_pivot yields an in-bounds position.
            if !is_less(p, unsafe { v.get_unchecked(pivot_pos) }) {
                let mid = partition(v, pivot_pos, &mut |a, b| !is_less(b, a));

                // Continue sorting elements greater than the pivot. We know that mid contains the
                // pivot. So we can continue after mid.
                v = &mut v[(mid + 1)..];
                ancestor_pivot = None;
                continue;
            }
        }

        // Partition the slice.
        let mid = partition(v, pivot_pos, is_less);
        // SAFETY: partition ensures that `mid` will be in-bounds.
        unsafe { intrinsics::assume(mid < v.len()) };

        // Split the slice into `left`, `pivot`, and `right`.
        let (left, right) = v.split_at_mut(mid);
        let (pivot, right) = right.split_at_mut(1);
        let pivot = &pivot[0];

        // Recurse into the left side. We have a fixed recursion limit, testing shows no real
        // benefit for recursing into the shorter side.
        quicksort(left, is_less, ancestor_pivot, limit);

        // Continue with the right side.
        v = right;
        ancestor_pivot = Some(pivot);
    }
}

// TODO move to main docs.
// Instead of swapping one pair at the time, it is more efficient to perform a cyclic
// permutation. This is not strictly equivalent to swapping, but produces a similar
// result using fewer memory operations.
//
// Example cyclic permutation to swap A,B,C,D with W,X,Y,Z
//
// A -> TMP
// Z -> A   | Z,B,C,D ___ W,X,Y,Z
//
// Loop iter 1
// B -> Z   | Z,B,C,D ___ W,X,Y,B
// Y -> B   | Z,Y,C,D ___ W,X,Y,B
//
// Loop iter 2
// C -> Y   | Z,Y,C,D ___ W,X,C,B
// X -> C   | Z,Y,X,D ___ W,X,C,B
//
// Loop iter 3
// D -> X   | Z,Y,X,D ___ W,D,C,B
// W -> D   | Z,Y,X,W ___ W,D,C,B
//
// TMP -> W | Z,Y,X,W ___ A,D,C,B

/// Takes the input slice `v` and re-arranges elements such that when the call returns normally
/// all elements that compare true for `is_less(elem, pivot)` where `pivot == v[pivot_pos]` are
/// on the left side of `v` followed by the other elements, notionally considered greater or
/// equal to `pivot`.
///
/// Returns the number of elements that are compared true for `is_less(elem, pivot)`.
///
/// If `is_less` does not implement a total order the resulting order and return value are
/// unspecified. All original elements will remain in `v` and any possible modifications via
/// interior mutability will be observable. Same is true if `is_less` panics or `v.len()`
/// exceeds `scratch.len()`.
fn partition<T, F>(v: &mut [T], pivot: usize, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    // Allows for panic-free code-gen by proving this property to the compiler.
    if len == 0 {
        return 0;
    }

    // Allows for panic-free code-gen by proving this property to the compiler.
    if pivot >= len {
        intrinsics::abort();
    }

    // Place the pivot at the beginning of slice.
    v.swap(0, pivot);
    let (pivot, v_without_pivot) = v.split_at_mut(1);

    // Assuming that Rust generates noalias LLVM IR we can be sure that a partition function
    // signature of the form `(v: &mut [T], pivot: &T)` guarantees that pivot and v can't alias.
    // Having this guarantee is crucial for optimizations. It's possible to copy the pivot value
    // into a stack value, but this creates issues for types with interior mutability mandating
    // a drop guard.
    let pivot = &mut pivot[0];

    let lt_count = T::partition(v_without_pivot, pivot, is_less);

    // Place the pivot between the two partitions.
    v.swap(0, lt_count);

    lt_count
}

trait PartitionImpl: Sized {
    /// See [`partition`].
    fn partition<F>(v: &mut [Self], pivot: &Self, is_less: &mut F) -> usize
    where
        F: FnMut(&Self, &Self) -> bool;
}

impl<T> PartitionImpl for T {
    default fn partition<F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
    where
        F: FnMut(&T, &T) -> bool,
    {
        partition_hoare_branchy_cyclic(v, pivot, is_less)
    }
}

const MAX_BRANCHLESS_PARTITION_SIZE: usize = 96;

/// Specialize for types that are relatively cheap to copy, where branchless optimizations have
/// large leverage e.g. `u64` and `String`.
impl<T> PartitionImpl for T
where
    (): IsTrue<{ mem::size_of::<T>() <= MAX_BRANCHLESS_PARTITION_SIZE }>,
{
    fn partition<F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
    where
        F: FnMut(&T, &T) -> bool,
    {
        partition_lomuto_branchless_cyclic(v, pivot, is_less)
    }
}

/// See [`partition`].
fn partition_hoare_branchy_cyclic<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    if len == 0 {
        return 0;
    }

    // Optimized for large types that are expensive to move. Not optimized for integers. Optimized
    // for small code-gen, assuming that is_less is an expensive operation that generates
    // substantial amounts of code or a call. And that copying elements will likely be a call to
    // memcpy. Using 2 `ptr::copy_nonoverlapping` has the chance to be faster than
    // `ptr::swap_nonoverlapping` because `memcpy` can use wide SIMD based on runtime feature
    // detection. Benchmarks support this analysis.

    let mut gap_opt: Option<GapGuard<T>> = None;

    // SAFETY: The left-to-right scanning loop performs a bounds check, where we know that `left >=
    // v_base && left < right && right <= v_base.add(len)`. The right-to-left scanning loop performs
    // a bounds check ensuring that `right` is in-bounds. We checked that `len` is more than zero,
    // which means that unconditional `right = right.sub(1)` is safe to do. The exit check makes
    // sure that `left` and `right` never alias, making `ptr::copy_nonoverlapping` safe. The
    // drop-guard `gap` ensures that should `is_less` panic we always overwrite the duplicate in the
    // input. `gap.pos` stores the previous value of `right` and starts at `right` and so it too is
    // in-bounds. We never pass the saved `gap.value` to `is_less` while it is inside the `GapGuard`
    // thus any changes via interior mutability will be observed.
    unsafe {
        let v_base = v.as_mut_ptr();

        let mut left = v_base;
        let mut right = v_base.add(len);

        loop {
            // Find the first element greater than the pivot.
            while left < right && is_less(&*left, pivot) {
                left = left.add(1);
            }

            // Find the last element equal to the pivot.
            loop {
                right = right.sub(1);
                if left >= right || is_less(&*right, pivot) {
                    break;
                }
            }

            if left >= right {
                break;
            }

            // Swap the found pair of out-of-order elements via cyclic permutation.
            let is_first_swap_pair = gap_opt.is_none();

            if is_first_swap_pair {
                gap_opt = Some(GapGuard {
                    pos: right,
                    value: ManuallyDrop::new(ptr::read(left)),
                });
            }

            let gap = gap_opt.as_mut().unwrap_unchecked();

            // Single place where we instantiate ptr::copy_nonoverlapping in the partition.
            if !is_first_swap_pair {
                ptr::copy_nonoverlapping(left, gap.pos, 1);
            }
            gap.pos = right;
            ptr::copy_nonoverlapping(right, left, 1);

            left = left.add(1);
        }

        left.sub_ptr(v_base)

        // `gap_opt` goes out of scope and overwrites the last wrong-side element on the right side
        // with the first wrong-side element of the left side that was initially overwritten by the
        // first wrong-side element on the right side element.
    }
}

/// This construct works around a couple of issues with auto unrolling as well as manual unrolling.
/// Auto unrolling as tested with rustc 1.75 is somewhat run-time and binary-size inefficient,
/// because it performs additional math to calculate the loop end, which we can avoid by
/// precomputing the loop end. Also auto unrolling only happens on x86 but not on Arm where doing so
/// for the Firestorm micro-architecture yields a 5+% performance improvement. Manual unrolling via
/// repeated code has a large negative impact on debug compile-times, and unrolling via `for _ in
/// 0..UNROLL_LEN` has a 10-20% perf penalty when compiling with `opt-level=s` which is deemed
/// unacceptable for such a crucial component of the sort implementation.
trait UnrollHelper: Sized {
    const UNROLL_LEN: usize;

    unsafe fn unrolled_loop_body<F: FnMut()>(loop_body: F);
}

impl<T> UnrollHelper for T {
    default const UNROLL_LEN: usize = 1;

    default unsafe fn unrolled_loop_body<F: FnMut()>(mut loop_body: F) {
        loop_body();
    }
}

impl<T> UnrollHelper for T
where
    (): crate::IsTrue<{ mem::size_of::<T>() <= 16 }>,
{
    const UNROLL_LEN: usize = 2;

    unsafe fn unrolled_loop_body<F: FnMut()>(mut loop_body: F) {
        loop_body();
        loop_body();
    }
}

fn partition_lomuto_branchless_cyclic<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // Novel partition implementation by Lukas Bergdoll and Orson Peters. Branchless Lomuto
    // partition paired with a cyclic permutation. TODO link writeup.

    let len = v.len();
    let v_base = v.as_mut_ptr();

    if len == 0 {
        return 0;
    }

    // SAFETY: We checked that `len` is more than zero, which means that reading `v_base` is safe to
    // do. From there we have a bounded loop where `v_base.add(i)` is guaranteed in-bounds. `v` and
    // `pivot` can't alias because of type system rules. The drop-guard `gap` ensures that should
    // `is_less` panic we always overwrite the duplicate in the input. `gap.pos` stores the previous
    // value of `right` and starts at `v_base` and so it too is in-bounds. Given `UNROLL_LEN == 2`
    // after the main loop we either have A) the last element in `v` that has not yet been processed
    // because `len % 2 != 0`, or B) all elements have been processed except the gap value that was
    // saved at the beginning with `ptr::read(v_base)`. In the case A) the loop will iterate twice,
    // first performing loop_body to take care of the last element that didn't fit into the unroll.
    // After that the behavior is the same as for B) where we use the saved value as `right` to
    // overwrite the duplicate. If this very last call to `is_less` panics the saved value will be
    // copied back including all possible changes via interior mutability. If `is_less` does not
    // panic and the code continues we overwrite the duplicate and do `right = right.add(1)`, this
    // is safe to do with `&mut *gap.value` because `T` is the same as `[T; 1]` and generating a
    // pointer one past the allocation is safe.
    unsafe {
        let mut lt_count = 0;
        let mut right = v_base.add(1);

        let mut gap = GapGuard {
            pos: v_base,
            value: ManuallyDrop::new(ptr::read(v_base)),
        };

        macro_rules! loop_body {
            () => {{
                let right_is_lt = is_less(&*right, pivot);
                let left = v_base.add(lt_count);

                ptr::copy(left, gap.pos, 1);
                ptr::copy_nonoverlapping(right, left, 1);

                gap.pos = right;
                lt_count += right_is_lt as usize;

                right = right.add(1);
            }};
        }

        let unroll_end = v_base.add(len - (T::UNROLL_LEN - 1));
        while right < unroll_end {
            T::unrolled_loop_body(|| loop_body!());
        }

        let end = v_base.add(len);
        loop {
            let is_done = right == end;
            right = if is_done { &mut *gap.value } else { right };

            loop_body!();

            if is_done {
                mem::forget(gap);
                break;
            }
        }

        lt_count
    }
}
