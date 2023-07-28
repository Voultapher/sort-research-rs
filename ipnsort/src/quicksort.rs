use core::mem::{self, ManuallyDrop};
use core::ptr;

use crate::smallsort::SmallSortImpl;
use crate::{GapGuardNonoverlapping, GapGuardOverlapping, IsTrue};

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

        if v.len() <= T::MAX_SMALL_SORT_LEN {
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
    // Proves a bunch of useful stuff to the compiler.
    if v.len() == 0 {
        return 0;
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

    // type DebugT = i32;
    // let v_as_x = unsafe { mem::transmute::<&[T], &[DebugT]>(v_without_pivot) };
    // let pivot_as_x = unsafe { mem::transmute::<&T, &DebugT>(pivot) };

    // println!("pivot: {}", pivot_as_x);
    // println!("before: {v_as_x:?}");
    // let lt_count = <crate::other::partition::hoare_branchy_cyclic::PartitionImpl as crate::other::partition::Partition>::partition_by(v_without_pivot, pivot, is_less);
    // println!("after:  {v_as_x:?}");
    // println!("sub: {:?}\n", &v_as_x[..lt_count]);

    // for val in &v_as_x[lt_count..] {
    //     if val < pivot_as_x {
    //         println!("wrong val: {val}");
    //         panic!("partition impl is wrong");
    //     }
    // }

    let lt_count = T::partition(v_without_pivot, pivot, is_less);

    // let lt_count = <crate::other::partition::lomuto_branchless_cyclic_opt::PartitionImpl as crate::other::partition::Partition>::partition_by(v_without_pivot, pivot, is_less);

    // pivot quality measurement.
    // println!("len: {} is_less: {}", v.len(), l + lt_count);

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
    default fn partition<F>(v: &mut [Self], pivot: &Self, is_less: &mut F) -> usize
    where
        F: FnMut(&Self, &Self) -> bool,
    {
        partition_hoare_branchy_cyclic(v, pivot, is_less)
    }
}

/// Specialize for types that are relatively cheap to copy.
impl<T> PartitionImpl for T
where
    (): IsTrue<{ mem::size_of::<T>() <= 64 }>,
{
    fn partition<F>(v: &mut [Self], pivot: &Self, is_less: &mut F) -> usize
    where
        F: FnMut(&Self, &Self) -> bool,
    {
        partition_lomuto_branchless_cyclic(v, pivot, is_less)
    }
}

/// See [`partition`].
fn partition_hoare_branchy_cyclic<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // Optimized for large types that are expensive to move. Not optimized for integers. Optimized
    // for small code-gen, assuming that is_less is an expensive operation that generates
    // substantial amounts of code or a call. And that copying elements will likely be a call to
    // memcpy. Using 2 `ptr::copy_nonoverlapping` has the chance to be faster than
    // `ptr::swap_nonoverlapping` because `memcpy` can use wide SIMD based on runtime feature
    // detection. Benchmarks support this analysis.

    let mut gap_guard_opt: Option<GapGuardNonoverlapping<T>> = None;

    // SAFETY: The unsafety below involves indexing an array. For the first one: We already do
    // the bounds checking here with `l < r`. For the second one: We initially have `l == 0` and
    // `r == v.len()` and we checked that `l < r` at every indexing operation.
    //
    // From here we know that `r` must be at least `r == l` which was shown to be valid from the
    // first one.
    unsafe {
        let arr_ptr = v.as_mut_ptr();

        let mut l_ptr = arr_ptr;
        let mut r_ptr = arr_ptr.add(v.len());

        loop {
            // Find the first element greater than the pivot.
            while l_ptr < r_ptr && is_less(&*l_ptr, pivot) {
                l_ptr = l_ptr.add(1);
            }

            // Find the last element equal to the pivot.
            while l_ptr < r_ptr && !is_less(&*r_ptr.sub(1), pivot) {
                r_ptr = r_ptr.sub(1);
            }
            r_ptr = r_ptr.sub(1);

            // Are we done?
            if l_ptr >= r_ptr {
                break;
            }

            // Swap the found pair of out-of-order elements via cyclic permutation.
            let is_first_swap_pair = gap_guard_opt.is_none();

            if is_first_swap_pair {
                gap_guard_opt = Some(GapGuardNonoverlapping {
                    pos: r_ptr,
                    value: ManuallyDrop::new(ptr::read(l_ptr)),
                });
            }

            let gap_guard = gap_guard_opt.as_mut().unwrap_unchecked();

            // Single place where we instantiate ptr::copy_nonoverlapping in the partition.
            if !is_first_swap_pair {
                ptr::copy_nonoverlapping(l_ptr, gap_guard.pos, 1);
            }
            gap_guard.pos = r_ptr;
            ptr::copy_nonoverlapping(r_ptr, l_ptr, 1);

            l_ptr = l_ptr.add(1);
        }

        l_ptr.sub_ptr(arr_ptr)

        // `gap_guard_opt` goes out of scope and overwrites the last right wrong-side element with
        // the first left wrong-side element that was initially overwritten by the first right
        // wrong-side element.
    }
}

fn partition_lomuto_branchless_cyclic<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // A Kind of branchless Lomuto partition paired with a cyclic permutation. As far as I can tell
    // this is a novel idea, developed by the author Lukas Bergdoll. Refined code-gen by Orson
    // Peters to avoid the cmov.

    // Manually unrolled to ensure consistent performance across various targets.
    const UNROLL_LEN: usize = 2;

    let len = v.len();
    if len == 0 {
        return 0;
    }

    unsafe {
        let arr_ptr = v.as_mut_ptr();

        let mut gap = GapGuardOverlapping {
            pos: arr_ptr,
            value: ManuallyDrop::new(ptr::read(arr_ptr)),
        };

        let end = arr_ptr.add(len);
        let mut lt_count = 0;
        while gap.pos.wrapping_add(UNROLL_LEN) < end {
            for _ in 0..UNROLL_LEN {
                let lt_ptr = arr_ptr.add(lt_count);
                let next_gap_pos = gap.pos.add(1);
                let is_next_lt = is_less(&*next_gap_pos, pivot);

                ptr::copy(lt_ptr, gap.pos, 1);
                ptr::copy_nonoverlapping(next_gap_pos, lt_ptr, 1);

                gap.pos = next_gap_pos;
                lt_count += is_next_lt as usize;
            }
        }

        let mut scan = gap.pos;
        drop(gap);

        while scan < end {
            let is_lomuto_less = is_less(&*scan, pivot);
            ptr::swap(arr_ptr.add(lt_count), scan);
            scan = scan.add(1);
            lt_count += is_lomuto_less as usize;
        }

        lt_count
    }
}
