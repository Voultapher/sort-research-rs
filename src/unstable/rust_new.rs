#![allow(unused)]

//! Slice sorting
//!
//! This module contains a sorting algorithm based on Orson Peters' pattern-defeating quicksort,
//! published at: <https://github.com/orlp/pdqsort>
//!
//! Unstable sorting is compatible with libcore because it doesn't allocate memory, unlike our
//! stable sorting implementation.

use std::cmp;
use std::cmp::Ordering;
use std::intrinsics;
use std::mem::{self, MaybeUninit};
use std::ptr;
use std::slice;

sort_impl!("rust_new_unstable");

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
#[inline]
pub fn sort<T>(arr: &mut [T])
where
    T: Ord,
{
    quicksort(arr, |a, b| a.lt(b));
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
#[inline]
pub fn sort_by<T, F>(arr: &mut [T], mut compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    quicksort(arr, |a, b| compare(a, b) == Ordering::Less);
}

// --- IMPL ---

/// Partially sorts a slice by shifting several out-of-order elements around.
///
/// Returns `true` if the slice is sorted at the end. This function is *O*(*n*) worst-case.
#[cold]
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partial_insertion_sort<T, F>(v: &mut [T], is_less: &mut F) -> bool
where
    F: FnMut(&T, &T) -> bool,
{
    // Maximum number of adjacent out-of-order pairs that will get shifted.
    const MAX_STEPS: usize = 5;
    // If the slice is shorter than this, don't shift any elements.
    const SHORTEST_SHIFTING: usize = 50;

    let len = v.len();
    let mut i = 1;

    for _ in 0..MAX_STEPS {
        // SAFETY: We already explicitly did the bound checking with `i < len`.
        // All our subsequent indexing is only in the range `0 <= index < len`
        unsafe {
            // Find the next pair of adjacent out-of-order elements.
            while i < len && !is_less(v.get_unchecked(i), v.get_unchecked(i - 1)) {
                i += 1;
            }
        }

        // Are we done?
        if i == len {
            return true;
        }

        // Don't shift elements on short arrays, that has a performance cost.
        if len < SHORTEST_SHIFTING {
            return false;
        }

        // Swap the found pair of elements. This puts them in correct order.
        v.swap(i - 1, i);

        if i >= 2 {
            // SAFETY: We check the that the slice len is >= 2.
            unsafe {
                insert_tail(&mut v[..i], is_less);
            }
        }

        // Shift the smaller element to the left.
        if i >= 2 {
            // SAFETY: We check the that the slice len is >= 2.
            unsafe {
                insert_tail(&mut v[..i], is_less);
            }
        }

        // Shift the greater element to the right.
        if i < (len - 1) {
            // SAFETY: We check the that the slice len is >= 2.
            unsafe {
                // shift_head(&mut v[i..], is_less);
                insert_head(&mut v[i..], is_less);
            }
        }
    }

    // Didn't manage to sort the slice in the limited number of steps.
    false
}

/// Sorts `v` using heapsort, which guarantees *O*(*n* \* log(*n*)) worst-case.
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
pub fn heapsort<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // This binary heap respects the invariant `parent >= child`.
    let mut sift_down = |v: &mut [T], mut node| {
        loop {
            // Children of `node`.
            let mut child = 2 * node + 1;
            if child >= v.len() {
                break;
            }

            // Choose the greater child.
            if child + 1 < v.len() && is_less(&v[child], &v[child + 1]) {
                child += 1;
            }

            // Stop if the invariant holds at `node`.
            if !is_less(&v[node], &v[child]) {
                break;
            }

            // Swap `node` with the greater child, move one step down, and continue sifting.
            v.swap(node, child);
            node = child;
        }
    };

    // Build the heap in linear time.
    for i in (0..v.len() / 2).rev() {
        sift_down(v, i);
    }

    // Pop maximal elements from the heap.
    for i in (1..v.len()).rev() {
        v.swap(0, i);
        sift_down(&mut v[..i], 0);
    }
}

/// Partitions `v` into elements smaller than `pivot`, followed by elements greater than or equal
/// to `pivot`.
///
/// Returns the number of elements smaller than `pivot`.
///
/// Partitioning is performed block-by-block in order to minimize the cost of branching operations.
/// This idea is presented in the [BlockQuicksort][pdf] paper.
///
/// [pdf]: https://drops.dagstuhl.de/opus/volltexte/2016/6389/pdf/LIPIcs-ESA-2016-38.pdf
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition_in_blocks<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // Number of elements in a typical block.
    const BLOCK: usize = u8::MAX as usize;

    // The partitioning algorithm repeats the following steps until completion:
    //
    // 1. Trace a block from the left side to identify elements greater than or equal to the pivot.
    // 2. Trace a block from the right side to identify elements smaller than the pivot.
    // 3. Exchange the identified elements between the left and right side.
    //
    // We keep the following variables for a block of elements:
    //
    // 1. `block` - Number of elements in the block.
    // 2. `start` - Start pointer into the `offsets` array.
    // 3. `end` - End pointer into the `offsets` array.
    // 4. `offsets - Indices of out-of-order elements within the block.

    // The current block on the left side (from `l` to `l.add(block_l)`).
    let mut l = v.as_mut_ptr();
    let mut block_l = BLOCK;
    let mut start_l = ptr::null_mut();
    let mut end_l = ptr::null_mut();
    let mut offsets_l = [MaybeUninit::<u8>::uninit(); BLOCK];

    // The current block on the right side (from `r.sub(block_r)` to `r`).
    // SAFETY: The documentation for .add() specifically mention that `vec.as_ptr().add(vec.len())` is always safe`
    let mut r = unsafe { l.add(v.len()) };
    let mut block_r = BLOCK;
    let mut start_r = ptr::null_mut();
    let mut end_r = ptr::null_mut();
    let mut offsets_r = [MaybeUninit::<u8>::uninit(); BLOCK];

    // FIXME: When we get VLAs, try creating one array of length `min(v.len(), 2 * BLOCK)` rather
    // than two fixed-size arrays of length `BLOCK`. VLAs might be more cache-efficient.

    // Returns the number of elements between pointers `l` (inclusive) and `r` (exclusive).
    fn width<T>(l: *mut T, r: *mut T) -> usize {
        debug_assert!(r.addr() >= l.addr());

        unsafe { r.sub_ptr(l) }
    }

    loop {
        // We are done with partitioning block-by-block when `l` and `r` get very close. Then we do
        // some patch-up work in order to partition the remaining elements in between.
        let is_done = width(l, r) <= 2 * BLOCK;

        if is_done {
            // Number of remaining elements (still not compared to the pivot).
            let mut rem = width(l, r);
            if start_l < end_l || start_r < end_r {
                rem -= BLOCK;
            }

            // Adjust block sizes so that the left and right block don't overlap, but get perfectly
            // aligned to cover the whole remaining gap.
            if start_l < end_l {
                block_r = rem;
            } else if start_r < end_r {
                block_l = rem;
            } else {
                // There were the same number of elements to switch on both blocks during the last
                // iteration, so there are no remaining elements on either block. Cover the remaining
                // items with roughly equally-sized blocks.
                block_l = rem / 2;
                block_r = rem - block_l;
            }
            debug_assert!(block_l <= BLOCK && block_r <= BLOCK);
            debug_assert!(width(l, r) == block_l + block_r);
        }

        if start_l == end_l {
            // Trace `block_l` elements from the left side.
            start_l = MaybeUninit::slice_as_mut_ptr(&mut offsets_l);
            end_l = start_l;
            let mut elem = l;

            for i in 0..block_l {
                // SAFETY: The unsafety operations below involve the usage of the `offset`.
                //         According to the conditions required by the function, we satisfy them because:
                //         1. `offsets_l` is stack-allocated, and thus considered separate allocated object.
                //         2. The function `is_less` returns a `bool`.
                //            Casting a `bool` will never overflow `isize`.
                //         3. We have guaranteed that `block_l` will be `<= BLOCK`.
                //            Plus, `end_l` was initially set to the begin pointer of `offsets_` which was declared on the stack.
                //            Thus, we know that even in the worst case (all invocations of `is_less` returns false) we will only be at most 1 byte pass the end.
                //        Another unsafety operation here is dereferencing `elem`.
                //        However, `elem` was initially the begin pointer to the slice which is always valid.
                unsafe {
                    // Branchless comparison.
                    *end_l = i as u8;
                    end_l = end_l.wrapping_add(!is_less(&*elem, pivot) as usize);
                    elem = elem.add(1);
                }
            }
        }

        if start_r == end_r {
            // Trace `block_r` elements from the right side.
            start_r = MaybeUninit::slice_as_mut_ptr(&mut offsets_r);
            end_r = start_r;
            let mut elem = r;

            for i in 0..block_r {
                // SAFETY: The unsafety operations below involve the usage of the `offset`.
                //         According to the conditions required by the function, we satisfy them because:
                //         1. `offsets_r` is stack-allocated, and thus considered separate allocated object.
                //         2. The function `is_less` returns a `bool`.
                //            Casting a `bool` will never overflow `isize`.
                //         3. We have guaranteed that `block_r` will be `<= BLOCK`.
                //            Plus, `end_r` was initially set to the begin pointer of `offsets_` which was declared on the stack.
                //            Thus, we know that even in the worst case (all invocations of `is_less` returns true) we will only be at most 1 byte pass the end.
                //        Another unsafety operation here is dereferencing `elem`.
                //        However, `elem` was initially `1 * sizeof(T)` past the end and we decrement it by `1 * sizeof(T)` before accessing it.
                //        Plus, `block_r` was asserted to be less than `BLOCK` and `elem` will therefore at most be pointing to the beginning of the slice.
                unsafe {
                    // Branchless comparison.
                    elem = elem.sub(1);
                    *end_r = i as u8;
                    end_r = end_r.wrapping_add(is_less(&*elem, pivot) as usize);
                }
            }
        }

        // Number of out-of-order elements to swap between the left and right side.
        let count = cmp::min(width(start_l, end_l), width(start_r, end_r));

        if count > 0 {
            macro_rules! left {
                () => {
                    l.add(*start_l as usize)
                };
            }
            macro_rules! right {
                () => {
                    r.sub(*start_r as usize + 1)
                };
            }

            // Instead of swapping one pair at the time, it is more efficient to perform a cyclic
            // permutation. This is not strictly equivalent to swapping, but produces a similar
            // result using fewer memory operations.

            // SAFETY: The use of `ptr::read` is valid because there is at least one element in
            // both `offsets_l` and `offsets_r`, so `left!` is a valid pointer to read from.
            //
            // The uses of `left!` involve calls to `offset` on `l`, which points to the
            // beginning of `v`. All the offsets pointed-to by `start_l` are at most `block_l`, so
            // these `offset` calls are safe as all reads are within the block. The same argument
            // applies for the uses of `right!`.
            //
            // The calls to `start_l.offset` are valid because there are at most `count-1` of them,
            // plus the final one at the end of the unsafe block, where `count` is the minimum number
            // of collected offsets in `offsets_l` and `offsets_r`, so there is no risk of there not
            // being enough elements. The same reasoning applies to the calls to `start_r.offset`.
            //
            // The calls to `copy_nonoverlapping` are safe because `left!` and `right!` are guaranteed
            // not to overlap, and are valid because of the reasoning above.
            unsafe {
                let tmp = ptr::read(left!());
                ptr::copy_nonoverlapping(right!(), left!(), 1);

                for _ in 1..count {
                    start_l = start_l.add(1);
                    ptr::copy_nonoverlapping(left!(), right!(), 1);
                    start_r = start_r.add(1);
                    ptr::copy_nonoverlapping(right!(), left!(), 1);
                }

                ptr::copy_nonoverlapping(&tmp, right!(), 1);
                mem::forget(tmp);
                start_l = start_l.add(1);
                start_r = start_r.add(1);
            }
        }

        if start_l == end_l {
            // All out-of-order elements in the left block were moved. Move to the next block.

            // block-width-guarantee
            // SAFETY: if `!is_done` then the slice width is guaranteed to be at least `2*BLOCK` wide. There
            // are at most `BLOCK` elements in `offsets_l` because of its size, so the `offset` operation is
            // safe. Otherwise, the debug assertions in the `is_done` case guarantee that
            // `width(l, r) == block_l + block_r`, namely, that the block sizes have been adjusted to account
            // for the smaller number of remaining elements.
            l = unsafe { l.add(block_l) };
        }

        if start_r == end_r {
            // All out-of-order elements in the right block were moved. Move to the previous block.

            // SAFETY: Same argument as [block-width-guarantee]. Either this is a full block `2*BLOCK`-wide,
            // or `block_r` has been adjusted for the last handful of elements.
            r = unsafe { r.sub(block_r) };
        }

        if is_done {
            break;
        }
    }

    // All that remains now is at most one block (either the left or the right) with out-of-order
    // elements that need to be moved. Such remaining elements can be simply shifted to the end
    // within their block.

    if start_l < end_l {
        // The left block remains.
        // Move its remaining out-of-order elements to the far right.
        debug_assert_eq!(width(l, r), block_l);
        while start_l < end_l {
            // remaining-elements-safety
            // SAFETY: while the loop condition holds there are still elements in `offsets_l`, so it
            // is safe to point `end_l` to the previous element.
            //
            // The `ptr::swap` is safe if both its arguments are valid for reads and writes:
            //  - Per the debug assert above, the distance between `l` and `r` is `block_l`
            //    elements, so there can be at most `block_l` remaining offsets between `start_l`
            //    and `end_l`. This means `r` will be moved at most `block_l` steps back, which
            //    makes the `r.offset` calls valid (at that point `l == r`).
            //  - `offsets_l` contains valid offsets into `v` collected during the partitioning of
            //    the last block, so the `l.offset` calls are valid.
            unsafe {
                end_l = end_l.sub(1);
                ptr::swap(l.add(*end_l as usize), r.sub(1));
                r = r.sub(1);
            }
        }
        width(v.as_mut_ptr(), r)
    } else if start_r < end_r {
        // The right block remains.
        // Move its remaining out-of-order elements to the far left.
        debug_assert_eq!(width(l, r), block_r);
        while start_r < end_r {
            // SAFETY: See the reasoning in [remaining-elements-safety].
            unsafe {
                end_r = end_r.sub(1);
                ptr::swap(l, r.sub(*end_r as usize + 1));
                l = l.add(1);
            }
        }
        width(v.as_mut_ptr(), l)
    } else {
        // Nothing else to do, we're done.
        width(v.as_mut_ptr(), l)
    }
}

/// Partitions `v` into elements smaller than `v[pivot]`, followed by elements greater than or
/// equal to `v[pivot]`.
///
/// Returns a tuple of:
///
/// 1. Number of elements smaller than `v[pivot]`.
/// 2. True if `v` was already partitioned.
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F>(v: &mut [T], pivot: usize, is_less: &mut F) -> (usize, bool)
where
    F: FnMut(&T, &T) -> bool,
{
    let (mid, was_partitioned) = {
        // Place the pivot at the beginning of slice.
        v.swap(0, pivot);
        let (pivot, v) = v.split_at_mut(1);
        let pivot = &mut pivot[0];

        // Read the pivot into a stack-allocated variable for efficiency. If a following comparison
        // operation panics, the pivot will be automatically written back into the slice.

        // SAFETY: `pivot` is a reference to the first element of `v`, so `ptr::read` is safe.
        let tmp = mem::ManuallyDrop::new(unsafe { ptr::read(pivot) });
        let _pivot_guard = InsertionHole {
            src: &*tmp,
            dest: pivot,
        };
        let pivot = &*tmp;

        // Find the first pair of out-of-order elements.
        let mut l = 0;
        let mut r = v.len();

        // SAFETY: The unsafety below involves indexing an array.
        // For the first one: We already do the bounds checking here with `l < r`.
        // For the second one: We initially have `l == 0` and `r == v.len()` and we checked that `l < r` at every indexing operation.
        //                     From here we know that `r` must be at least `r == l` which was shown to be valid from the first one.
        unsafe {
            // This search introduces a non-trivial amount of branch mis-prediction. And less work
            // can be done in the faster partition_in_blocks. It is only useful for larger sizes
            // where was_partitioned is relevant and likely. It's hard to quantify the total impact
            // of this, across various benchmarks I saw better performance in some cases and never
            // worse performance.
            if v.len() > ((max_len_small_sort::<T>() * 3) / 2) {
                // Find the first element greater than or equal to the pivot.
                while l < r && is_less(v.get_unchecked(l), pivot) {
                    l += 1;
                }

                // Find the last element smaller that the pivot.
                while l < r && !is_less(v.get_unchecked(r - 1), pivot) {
                    r -= 1;
                }
            }
        }

        (
            // l + <crate::other::partition::new_block_quicksort::PartitionImpl as crate::other::partition::Partition>::partition_by(&mut v[l..r], pivot, is_less),
            l + partition_in_blocks(&mut v[l..r], pivot, is_less),
            l >= r,
        )

        // `_pivot_guard` goes out of scope and writes the pivot (which is a stack-allocated
        // variable) back into the slice where it originally was. This step is critical in ensuring
        // safety!
    };

    // Place the pivot between the two partitions.
    v.swap(0, mid);

    (mid, was_partitioned)
}

/// Partitions `v` into elements equal to `v[pivot]` followed by elements greater than `v[pivot]`.
///
/// Returns the number of elements equal to the pivot. It is assumed that `v` does not contain
/// elements smaller than the pivot.
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition_equal<T, F>(v: &mut [T], pivot: usize, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    partition(v, pivot, &mut |a, b| !is_less(b, a)).0
}

/// Scatters some elements around in an attempt to break patterns that might cause imbalanced
/// partitions in quicksort.
#[cold]
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn break_patterns<T>(v: &mut [T]) {
    let len = v.len();
    if len >= 8 {
        // Pseudorandom number generator from the "Xorshift RNGs" paper by George Marsaglia.
        let mut random = len as u32;
        let mut gen_u32 = || {
            random ^= random << 13;
            random ^= random >> 17;
            random ^= random << 5;
            random
        };
        let mut gen_usize = || {
            if usize::BITS <= 32 {
                gen_u32() as usize
            } else {
                (((gen_u32() as u64) << 32) | (gen_u32() as u64)) as usize
            }
        };

        // Take random numbers modulo this number.
        // The number fits into `usize` because `len` is not greater than `isize::MAX`.
        let modulus = len.next_power_of_two();

        // Some pivot candidates will be in the nearby of this index. Let's randomize them.
        let pos = len / 4 * 2;

        for i in 0..3 {
            // Generate a random number modulo `len`. However, in order to avoid costly operations
            // we first take it modulo a power of two, and then decrease by `len` until it fits
            // into the range `[0, len - 1]`.
            let mut other = gen_usize() & (modulus - 1);

            // `other` is guaranteed to be less than `2 * len`.
            if other >= len {
                other -= len;
            }

            v.swap(pos - 1 + i, other);
        }
    }
}

/// Chooses a pivot in `v` and returns the index and `true` if the slice is likely already sorted.
///
/// Elements in `v` might be reordered in the process.
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn choose_pivot<T, F>(v: &mut [T], is_less: &mut F) -> (usize, bool)
where
    F: FnMut(&T, &T) -> bool,
{
    // Minimum length to choose the median-of-medians method.
    // Shorter slices use the simple median-of-three method.
    const SHORTEST_MEDIAN_OF_MEDIANS: usize = 50;
    // Maximum number of swaps that can be performed in this function.
    const MAX_SWAPS: usize = 4 * 3;

    let len = v.len();

    if len <= max_len_small_sort::<T>() {
        // It's a logic bug if this get's called on slice that would be small-sorted.
        debug_assert!(false);
        return (0, false);
    }

    // Three indices near which we are going to choose a pivot.
    let len_div_4 = len / 4;
    let mut a = len_div_4 * 1;
    let mut b = len_div_4 * 2;
    let mut c = len_div_4 * 3;

    // Counts the total number of swaps we are about to perform while sorting indices.
    let mut swaps = 0;

    // Swaps indices so that `v[a] <= v[b]`.
    // SAFETY: `len > 20` so there are at least two elements in the neighborhoods of
    // `a`, `b` and `c`. This means the three calls to `sort_adjacent` result in
    // corresponding calls to `sort3` with valid 3-item neighborhoods around each
    // pointer, which in turn means the calls to `sort2` are done with valid
    // references. Thus the `v.get_unchecked` calls are safe, as is the `ptr::swap`
    // call.
    let mut sort2_idx = |a: &mut usize, b: &mut usize| unsafe {
        let should_swap = is_less(v.get_unchecked(*b), v.get_unchecked(*a));

        // Generate branchless cmov code, it's not super important but reduces BHB and BTB pressure.
        let tmp_idx = if should_swap { *a } else { *b };
        *a = if should_swap { *b } else { *a };
        *b = tmp_idx;
        swaps += should_swap as usize;
    };

    // Swaps indices so that `v[a] <= v[b] <= v[c]`.
    let mut sort3_idx = |a: &mut usize, b: &mut usize, c: &mut usize| {
        sort2_idx(a, b);
        sort2_idx(b, c);
        sort2_idx(a, b);
    };

    if len >= SHORTEST_MEDIAN_OF_MEDIANS {
        // Finds the median of `v[a - 1], v[a], v[a + 1]` and stores the index into `a`.
        let mut sort_adjacent = |a: &mut usize| {
            let tmp = *a;
            sort3_idx(&mut (tmp - 1), a, &mut (tmp + 1));
        };

        // Find medians in the neighborhoods of `a`, `b`, and `c`.
        sort_adjacent(&mut a);
        sort_adjacent(&mut b);
        sort_adjacent(&mut c);
    }

    // Find the median among `a`, `b`, and `c`.
    sort3_idx(&mut a, &mut b, &mut c);

    let left_len = b;
    let right_len = len - b;

    if swaps < MAX_SWAPS {
        (b, swaps == 0)
    } else {
        // The maximum number of swaps was performed. Chances are the slice is descending or mostly
        // descending, so reversing will probably help sort it faster.
        v.reverse();
        (len - 1 - b, true)
    }
}

/// Sorts `v` recursively.
///
/// If the slice had a predecessor in the original array, it is specified as `pred`.
///
/// `limit` is the number of allowed imbalanced partitions before switching to `heapsort`. If zero,
/// this function will immediately switch to heapsort.
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn recurse<'a, T, F>(mut v: &'a mut [T], is_less: &mut F, mut pred: Option<&'a T>, mut limit: u32)
where
    F: FnMut(&T, &T) -> bool,
{
    // True if the last partitioning was reasonably balanced.
    let mut was_balanced = true;
    // True if the last partitioning didn't shuffle elements (the slice was already partitioned).
    let mut was_partitioned = true;

    loop {
        let len = v.len();

        // println!("len: {len}");

        if sort_small(v, is_less) {
            return;
        }

        // If too many bad pivot choices were made, simply fall back to heapsort in order to
        // guarantee `O(n * log(n))` worst-case.
        if limit == 0 {
            heapsort(v, is_less);
            return;
        }

        // If the last partitioning was imbalanced, try breaking patterns in the slice by shuffling
        // some elements around. Hopefully we'll choose a better pivot this time.
        if !was_balanced {
            break_patterns(v);
            limit -= 1;
        }

        // Choose a pivot and try guessing whether the slice is already sorted.
        let (pivot, likely_sorted) = choose_pivot(v, is_less);

        // If the last partitioning was decently balanced and didn't shuffle elements, and if pivot
        // selection predicts the slice is likely already sorted...
        if was_balanced && was_partitioned && likely_sorted {
            // Try identifying several out-of-order elements and shifting them to correct
            // positions. If the slice ends up being completely sorted, we're done.
            if partial_insertion_sort(v, is_less) {
                return;
            }
        }

        // If the chosen pivot is equal to the predecessor, then it's the smallest element in the
        // slice. Partition the slice into elements equal to and elements greater than the pivot.
        // This case is usually hit when the slice contains many duplicate elements.
        if let Some(p) = pred {
            if !is_less(p, &v[pivot]) {
                let mid = partition_equal(v, pivot, is_less);

                // Continue sorting elements greater than the pivot.
                v = &mut v[mid..];
                continue;
            }
        }

        // Partition the slice.
        let (mid, was_p) = partition(v, pivot, is_less);
        was_balanced = cmp::min(mid, len - mid) >= len / 8;
        was_partitioned = was_p;

        // Split the slice into `left`, `pivot`, and `right`.
        let (left, right) = v.split_at_mut(mid);
        let (pivot, right) = right.split_at_mut(1);
        let pivot = &pivot[0];

        // Recurse into the shorter side only in order to minimize the total number of recursive
        // calls and consume less stack space. Then just continue with the longer side (this is
        // akin to tail recursion).
        if left.len() < right.len() {
            recurse(left, is_less, pred, limit);
            v = right;
            pred = Some(pivot);
        } else {
            recurse(right, is_less, Some(pivot), limit);
            v = left;
        }
    }
}

/// Finds a streak of presorted elements starting at the end of the slice.
/// Returns the first value that is not part of said streak.
/// Streaks can be increasing or decreasing.
/// Decreasing streaks will be reversed.
/// After this call `v[start..len]` will be sorted.
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn find_streak_rev<T, F>(v: &mut [T], is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    let mut start = len - 1;
    if start > 0 {
        start -= 1;
        unsafe {
            if is_less(v.get_unchecked(start + 1), v.get_unchecked(start)) {
                while start > 0 && is_less(v.get_unchecked(start), v.get_unchecked(start - 1)) {
                    start -= 1;
                }
                v[start..len].reverse();
            } else {
                while start > 0 && !is_less(v.get_unchecked(start), v.get_unchecked(start - 1)) {
                    start -= 1;
                }
            }
        }
    }

    debug_assert!(start < len);

    start
}

/// Sorts `v` using strategies optimized for small sizes.
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn sort_small<T, F>(v: &mut [T], is_less: &mut F) -> bool
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    if len > max_len_small_sort::<T>() {
        return false;
    }

    if is_cheap_to_move::<T>() {
        // Always sort assuming somewhat random distribution.
        // Patterns should have already been found by the other analysis steps.
        //
        // Small total slices are handled separately, see function quicksort.
        if len >= 16 {
            sort16_plus(v, is_less);
        } else if len >= 8 {
            sort8_plus(v, is_less);
        } else if len >= 4 {
            sort4_plus(v, is_less);
        } else if len >= 2 {
            insertion_sort_shift_left(v, 1, is_less);
        }
    } else if len >= 2 {
        insertion_sort_shift_left(v, 1, is_less);
    }

    true
}

/// Sorts `v` using strategies optimized for small sizes.
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn sort_small_with_pattern_analysis<T, F>(v: &mut [T], is_less: &mut F) -> bool
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    assert!(len <= max_len_small_sort::<T>());

    if len < 2 {
        return true;
    }

    // Handles both ascending and descending inputs in N-1 comparisons.
    let start = find_streak_rev(v, is_less);
    if start == 0 {
        return true;
    }

    if !is_cheap_to_move::<T>() || start < cmp::min(len / 2, 8) {
        insertion_sort_shift_right(v, start, is_less);
        return true;
    }

    // For some reasons calling sort_small here has a 5+% perf overhead.
    // Delegate it by returning a bool, this also improves binary size.
    false
}

/// Sorts `v` using pattern-defeating quicksort, which is *O*(*n* \* log(*n*)) worst-case.
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
pub fn quicksort<T, F>(v: &mut [T], mut is_less: F)
where
    F: FnMut(&T, &T) -> bool,
{
    // Sorting has no meaningful behavior on zero-sized types.
    if mem::size_of::<T>() == 0 {
        return;
    }

    if v.len() <= max_len_small_sort::<T>() {
        // This code cannot rely on external pattern analysis,
        // so add extra code for already sorted inputs.
        if sort_small_with_pattern_analysis(v, &mut is_less) {
            return;
        }
    }

    // Limit the number of imbalanced partitions to `floor(log2(len)) + 1`.
    let limit = usize::BITS - v.len().leading_zeros();

    recurse(v, &mut is_less, None, limit);
}

// --- Insertion sorts ---

// TODO merge with local variants

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

/// Inserts `v[0]` into pre-sorted sequence `v[1..]` so that whole `v[..]` becomes sorted.
///
/// This is the integral subroutine of insertion sort.
unsafe fn insert_head<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    debug_assert!(v.len() >= 2);

    unsafe {
        if is_less(v.get_unchecked(1), v.get_unchecked(0)) {
            let arr_ptr = v.as_mut_ptr();

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
            let tmp = mem::ManuallyDrop::new(ptr::read(arr_ptr));

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
                dest: arr_ptr.add(1),
            };
            ptr::copy_nonoverlapping(arr_ptr.add(1), arr_ptr.add(0), 1);

            for i in 2..v.len() {
                if !is_less(&v.get_unchecked(i), &*tmp) {
                    break;
                }
                ptr::copy_nonoverlapping(arr_ptr.add(i), arr_ptr.add(i - 1), 1);
                hole.dest = arr_ptr.add(i);
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

/// Sort `v` assuming `v[offset..]` is already sorted.
///
/// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
/// performance impact. Even improving performance in some cases.
#[inline(never)]
fn insertion_sort_shift_right<T, F>(v: &mut [T], offset: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    // This would be a logic bug.
    // Using assert here improves performance.
    assert!(offset != 0 && offset <= len);

    // Shift each element of the unsorted region v[..i] as far left as is needed to make v sorted.
    for i in (0..offset).rev() {
        // We ensured that the slice length is always at least 2 long.
        // We know that start_found will be at least one less than end,
        // and the range is exclusive. Which gives us i always <= (end - 2).
        unsafe {
            insert_head(&mut v[i..len], is_less);
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

// Adapted from crumsort/quadsort.
unsafe fn parity_merge<T, F>(v: &[T], dest_ptr: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: the caller must guarantee that `dest_ptr` is valid for v.len() writes.
    // Also `v.as_ptr` and `dest_ptr` must not alias.
    //
    // The caller must guarantee that T cannot modify itself inside is_less.
    let len = v.len();
    let src_ptr = v.as_ptr();

    let block = len / 2;

    let mut ptr_left = src_ptr;
    let mut ptr_right = src_ptr.wrapping_add(block);
    let mut ptr_data = dest_ptr;

    let mut t_ptr_left = src_ptr.wrapping_add(block - 1);
    let mut t_ptr_right = src_ptr.wrapping_add(len - 1);
    let mut t_ptr_data = dest_ptr.wrapping_add(len - 1);

    for _ in 0..block {
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

// Slices of up to this length get sorted using optimized sorting for small slices.
const fn max_len_small_sort<T>() -> usize {
    if is_cheap_to_move::<T>() {
        40
    } else {
        20
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

// --- Branchless sorting (less branches not zero) ---

/// Swap two values in array pointed to by a_ptr and b_ptr if b is less than a.
#[inline]
pub unsafe fn branchless_swap<T>(a_ptr: *mut T, b_ptr: *mut T, should_swap: bool) {
    // SAFETY: the caller must guarantee that `a_ptr` and `b_ptr` are valid for writes
    // and properly aligned, and part of the same allocation, and do not alias.

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

    ptr::copy_nonoverlapping(b_swap_ptr, tmp.as_mut_ptr(), 1);
    ptr::copy(a_swap_ptr, a_ptr, 1);
    ptr::copy_nonoverlapping(tmp.as_ptr(), b_ptr, 1);
}

/// Swap two values in array pointed to by a_ptr and b_ptr if b is less than a.
#[inline]
pub unsafe fn swap_if_less<T, F>(arr_ptr: *mut T, a: usize, b: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: the caller must guarantee that `a` and `b` each added to `arr_ptr` yield valid
    // pointers into `arr_ptr`, and are properly aligned, and part of the same allocation, and do
    // not alias. `a` and `b` must be different numbers.
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

// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
// performance impact.
#[inline(never)]
unsafe fn sort4_optimal<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 4.
    debug_assert!(v.len() == 4);

    let arr_ptr = v.as_mut_ptr();

    // Optimal sorting network see:
    // https://bertdobbelaere.github.io/sorting_networks.html.

    swap_if_less(arr_ptr, 0, 2, is_less);
    swap_if_less(arr_ptr, 1, 3, is_less);
    swap_if_less(arr_ptr, 0, 1, is_less);
    swap_if_less(arr_ptr, 2, 3, is_less);
    swap_if_less(arr_ptr, 1, 2, is_less);
}

// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
// performance impact.
#[inline(never)]
unsafe fn sort8_optimal<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 8.
    debug_assert!(v.len() == 8);

    let arr_ptr = v.as_mut_ptr();

    // Optimal sorting network see:
    // https://bertdobbelaere.github.io/sorting_networks.html.

    swap_if_less(arr_ptr, 0, 2, is_less);
    swap_if_less(arr_ptr, 1, 3, is_less);
    swap_if_less(arr_ptr, 4, 6, is_less);
    swap_if_less(arr_ptr, 5, 7, is_less);
    swap_if_less(arr_ptr, 0, 4, is_less);
    swap_if_less(arr_ptr, 1, 5, is_less);
    swap_if_less(arr_ptr, 2, 6, is_less);
    swap_if_less(arr_ptr, 3, 7, is_less);
    swap_if_less(arr_ptr, 0, 1, is_less);
    swap_if_less(arr_ptr, 2, 3, is_less);
    swap_if_less(arr_ptr, 4, 5, is_less);
    swap_if_less(arr_ptr, 6, 7, is_less);
    swap_if_less(arr_ptr, 2, 4, is_less);
    swap_if_less(arr_ptr, 3, 5, is_less);
    swap_if_less(arr_ptr, 1, 4, is_less);
    swap_if_less(arr_ptr, 3, 6, is_less);
    swap_if_less(arr_ptr, 1, 2, is_less);
    swap_if_less(arr_ptr, 3, 4, is_less);
    swap_if_less(arr_ptr, 5, 6, is_less);
}

// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
// performance impact.
#[inline(never)]
unsafe fn sort12_optimal<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // TODO check if replacing sort8 is worth it.

    // SAFETY: caller must ensure v.len() >= 12.
    debug_assert!(v.len() == 12);

    let arr_ptr = v.as_mut_ptr();

    // Optimal sorting network see:
    // https://bertdobbelaere.github.io/sorting_networks.html.

    swap_if_less(arr_ptr, 0, 8, is_less);
    swap_if_less(arr_ptr, 1, 7, is_less);
    swap_if_less(arr_ptr, 2, 6, is_less);
    swap_if_less(arr_ptr, 3, 11, is_less);
    swap_if_less(arr_ptr, 4, 10, is_less);
    swap_if_less(arr_ptr, 5, 9, is_less);
    swap_if_less(arr_ptr, 0, 1, is_less);
    swap_if_less(arr_ptr, 2, 5, is_less);
    swap_if_less(arr_ptr, 3, 4, is_less);
    swap_if_less(arr_ptr, 6, 9, is_less);
    swap_if_less(arr_ptr, 7, 8, is_less);
    swap_if_less(arr_ptr, 10, 11, is_less);
    swap_if_less(arr_ptr, 0, 2, is_less);
    swap_if_less(arr_ptr, 1, 6, is_less);
    swap_if_less(arr_ptr, 5, 10, is_less);
    swap_if_less(arr_ptr, 9, 11, is_less);
    swap_if_less(arr_ptr, 0, 3, is_less);
    swap_if_less(arr_ptr, 1, 2, is_less);
    swap_if_less(arr_ptr, 4, 6, is_less);
    swap_if_less(arr_ptr, 5, 7, is_less);
    swap_if_less(arr_ptr, 8, 11, is_less);
    swap_if_less(arr_ptr, 9, 10, is_less);
    swap_if_less(arr_ptr, 1, 4, is_less);
    swap_if_less(arr_ptr, 3, 5, is_less);
    swap_if_less(arr_ptr, 6, 8, is_less);
    swap_if_less(arr_ptr, 7, 10, is_less);
    swap_if_less(arr_ptr, 1, 3, is_less);
    swap_if_less(arr_ptr, 2, 5, is_less);
    swap_if_less(arr_ptr, 6, 9, is_less);
    swap_if_less(arr_ptr, 8, 10, is_less);
    swap_if_less(arr_ptr, 2, 3, is_less);
    swap_if_less(arr_ptr, 4, 5, is_less);
    swap_if_less(arr_ptr, 6, 7, is_less);
    swap_if_less(arr_ptr, 8, 9, is_less);
    swap_if_less(arr_ptr, 4, 6, is_less);
    swap_if_less(arr_ptr, 5, 7, is_less);
    swap_if_less(arr_ptr, 3, 4, is_less);
    swap_if_less(arr_ptr, 5, 6, is_less);
    swap_if_less(arr_ptr, 7, 8, is_less);
}

// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
// performance impact.
#[inline(never)]
unsafe fn sort16_optimal<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 16.
    debug_assert!(v.len() == 16);

    let arr_ptr = v.as_mut_ptr();

    // Optimal sorting network see:
    // https://bertdobbelaere.github.io/sorting_networks.html#N16L60D10

    swap_if_less(arr_ptr, 0, 13, is_less);
    swap_if_less(arr_ptr, 1, 12, is_less);
    swap_if_less(arr_ptr, 2, 15, is_less);
    swap_if_less(arr_ptr, 3, 14, is_less);
    swap_if_less(arr_ptr, 4, 8, is_less);
    swap_if_less(arr_ptr, 5, 6, is_less);
    swap_if_less(arr_ptr, 7, 11, is_less);
    swap_if_less(arr_ptr, 9, 10, is_less);
    swap_if_less(arr_ptr, 0, 5, is_less);
    swap_if_less(arr_ptr, 1, 7, is_less);
    swap_if_less(arr_ptr, 2, 9, is_less);
    swap_if_less(arr_ptr, 3, 4, is_less);
    swap_if_less(arr_ptr, 6, 13, is_less);
    swap_if_less(arr_ptr, 8, 14, is_less);
    swap_if_less(arr_ptr, 10, 15, is_less);
    swap_if_less(arr_ptr, 11, 12, is_less);
    swap_if_less(arr_ptr, 0, 1, is_less);
    swap_if_less(arr_ptr, 2, 3, is_less);
    swap_if_less(arr_ptr, 4, 5, is_less);
    swap_if_less(arr_ptr, 6, 8, is_less);
    swap_if_less(arr_ptr, 7, 9, is_less);
    swap_if_less(arr_ptr, 10, 11, is_less);
    swap_if_less(arr_ptr, 12, 13, is_less);
    swap_if_less(arr_ptr, 14, 15, is_less);
    swap_if_less(arr_ptr, 0, 2, is_less);
    swap_if_less(arr_ptr, 1, 3, is_less);
    swap_if_less(arr_ptr, 4, 10, is_less);
    swap_if_less(arr_ptr, 5, 11, is_less);
    swap_if_less(arr_ptr, 6, 7, is_less);
    swap_if_less(arr_ptr, 8, 9, is_less);
    swap_if_less(arr_ptr, 12, 14, is_less);
    swap_if_less(arr_ptr, 13, 15, is_less);
    swap_if_less(arr_ptr, 1, 2, is_less);
    swap_if_less(arr_ptr, 3, 12, is_less);
    swap_if_less(arr_ptr, 4, 6, is_less);
    swap_if_less(arr_ptr, 5, 7, is_less);
    swap_if_less(arr_ptr, 8, 10, is_less);
    swap_if_less(arr_ptr, 9, 11, is_less);
    swap_if_less(arr_ptr, 13, 14, is_less);
    swap_if_less(arr_ptr, 1, 4, is_less);
    swap_if_less(arr_ptr, 2, 6, is_less);
    swap_if_less(arr_ptr, 5, 8, is_less);
    swap_if_less(arr_ptr, 7, 10, is_less);
    swap_if_less(arr_ptr, 9, 13, is_less);
    swap_if_less(arr_ptr, 11, 14, is_less);
    swap_if_less(arr_ptr, 2, 4, is_less);
    swap_if_less(arr_ptr, 3, 6, is_less);
    swap_if_less(arr_ptr, 9, 12, is_less);
    swap_if_less(arr_ptr, 11, 13, is_less);
    swap_if_less(arr_ptr, 3, 5, is_less);
    swap_if_less(arr_ptr, 6, 8, is_less);
    swap_if_less(arr_ptr, 7, 9, is_less);
    swap_if_less(arr_ptr, 10, 12, is_less);
    swap_if_less(arr_ptr, 3, 4, is_less);
    swap_if_less(arr_ptr, 5, 6, is_less);
    swap_if_less(arr_ptr, 7, 8, is_less);
    swap_if_less(arr_ptr, 9, 10, is_less);
    swap_if_less(arr_ptr, 11, 12, is_less);
    swap_if_less(arr_ptr, 6, 7, is_less);
    swap_if_less(arr_ptr, 8, 9, is_less);
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn sort4_plus<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 4.
    let len = v.len();
    assert!(len >= 4);

    // SAFETY: We checked the len.
    unsafe {
        sort4_optimal(&mut v[0..4], is_less);
    }

    if len > 4 {
        insertion_sort_shift_left(v, 4, is_less);
    }
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn sort8_plus<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    assert!(len >= 8);

    // SAFETY: we check v.len() >= 8.
    unsafe {
        sort8_optimal(&mut v[0..8], is_less);
    }

    if len > 8 {
        insertion_sort_shift_left(&mut v[8..], 1, is_less);

        // SAFETY: We only need place for 8 entries because we know the shorter side is at most 8
        // long.
        unsafe {
            let mut swap = mem::MaybeUninit::<[T; 8]>::uninit();
            let swap_ptr = swap.as_mut_ptr() as *mut T;

            merge(v, 8, swap_ptr, is_less);
        }
    }
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn sort16_plus_parity<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 16.
    let len = v.len();
    const MAX_BRANCHLESS_SMALL_SORT: usize = max_len_small_sort::<i32>();

    assert!(len >= 16 && len <= MAX_BRANCHLESS_SMALL_SORT && T::is_copy());

    if len <= 23 {
        // SAFETY: we checked the len.
        unsafe {
            sort16_optimal(&mut v[0..16], is_less);
        }
        insertion_sort_shift_left(v, 16, is_less);
        return;
    }

    let even_len = len - (len % 2 != 0) as usize;
    let len_div_2 = even_len / 2; // TODO check optimization because we know it is even.

    let mid = if even_len <= 30 {
        // SAFETY: we checked the len.
        unsafe {
            sort12_optimal(&mut v[0..12], is_less);
            sort12_optimal(&mut v[len_div_2..(len_div_2 + 12)], is_less);
        }

        12
    } else {
        // SAFETY: we checked the len.
        unsafe {
            sort16_optimal(&mut v[0..16], is_less);
            sort16_optimal(&mut v[len_div_2..(len_div_2 + 16)], is_less);
        }

        16
    };

    insertion_sort_shift_left(&mut v[0..len_div_2], mid, is_less);
    insertion_sort_shift_left(&mut v[len_div_2..], mid, is_less);

    let mut swap = mem::MaybeUninit::<[T; MAX_BRANCHLESS_SMALL_SORT]>::uninit();
    let swap_ptr = swap.as_mut_ptr() as *mut T;

    // SAFETY: We checked that T is Copy and thus observation safe.
    // Should is_less panic v was not modified in parity_merge and retains it's original input.
    // swap and v must not alias and swap has v.len() space.
    unsafe {
        parity_merge(&mut v[..even_len], swap_ptr, is_less);
        ptr::copy_nonoverlapping(swap_ptr, v.as_mut_ptr(), even_len);
    }

    if len != even_len {
        // SAFETY: We know len >= 2.
        unsafe {
            insert_tail(v, is_less);
        }
    }
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn sort16_plus_min_cmp<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 16.
    let len = v.len();
    const MAX_BRANCHLESS_SMALL_SORT: usize = max_len_small_sort::<i32>();

    assert!(len >= 16 && len <= MAX_BRANCHLESS_SMALL_SORT);

    // SAFETY: We check the len.
    unsafe {
        sort16_optimal(&mut v[0..16], is_less);

        if len >= 17 {
            let start = match len {
                17..=19 => 1,
                20..=23 => {
                    sort4_optimal(&mut v[16..20], is_less);
                    4
                }
                24..=31 => {
                    sort8_optimal(&mut v[16..24], is_less);
                    8
                }
                32..=MAX_BRANCHLESS_SMALL_SORT => {
                    sort16_optimal(&mut v[16..32], is_less);
                    16
                }
                _ => {
                    unreachable!()
                }
            };

            insertion_sort_shift_left(&mut v[16..], start, is_less);

            // We only need place for 16 entries because we know the shorter side is at most 16 long.
            let mut swap = mem::MaybeUninit::<[T; 16]>::uninit();
            let swap_ptr = swap.as_mut_ptr() as *mut T;

            merge(v, 16, swap_ptr, is_less);
        }
    }
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn sort16_plus<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    if qualifies_for_parity_merge::<T>() {
        // Allows it to use more stack space.
        sort16_plus_parity(v, is_less);
    } else {
        // Minimize the amount of comparisons.
        sort16_plus_min_cmp(v, is_less);
    }
}
