use core::intrinsics;

use crate::smallsort::SmallSortImpl;

/// Sorts `v` recursively.
///
/// If the slice had a predecessor in the original array, it is specified as `ancestor_pivot`.
///
/// `limit` is the number of allowed imbalanced partitions before switching to `heapsort`. If zero,
/// this function will immediately switch to heapsort.
pub(crate) fn quicksort<'a, T, PF>(
    mut v: &'a mut [T],
    is_less: &mut impl FnMut(&T, &T) -> bool,
    mut ancestor_pivot: Option<&'a T>,
    mut limit: u32,
) where
    PF: sort_test_tools::Partition,
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
                let mid = partition::<T, PF>(v, pivot_pos, &mut |a, b| !is_less(b, a));

                // Continue sorting elements greater than the pivot. We know that mid contains the
                // pivot. So we can continue after mid.
                v = &mut v[(mid + 1)..];
                ancestor_pivot = None;
                continue;
            }
        }

        // Partition the slice.
        let mid = partition::<T, PF>(v, pivot_pos, is_less);
        // SAFETY: partition ensures that `mid` will be in-bounds.
        unsafe { intrinsics::assume(mid < v.len()) };

        // Split the slice into `left`, `pivot`, and `right`.
        let (left, right) = v.split_at_mut(mid);
        let (pivot, right) = right.split_at_mut(1);
        let pivot = &pivot[0];

        // Recurse into the left side. We have a fixed recursion limit, testing shows no real
        // benefit for recursing into the shorter side.
        quicksort::<T, PF>(left, is_less, ancestor_pivot, limit);

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
fn partition<T, PF>(v: &mut [T], pivot: usize, is_less: &mut impl FnMut(&T, &T) -> bool) -> usize
where
    PF: sort_test_tools::Partition,
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

    // let lt_count = T::partition(v_without_pivot, pivot, is_less);
    let num_lt = PF::partition_by(v_without_pivot, pivot, is_less);

    // Place the pivot between the two partitions.
    v.swap(0, num_lt);

    num_lt
}
