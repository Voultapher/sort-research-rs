#![allow(unused_unsafe)]

use core::cmp::Ordering;
use core::intrinsics;
use core::mem::{self, ManuallyDrop, SizedTypeProperties};
use core::ptr;

sort_impl!("rust_ipnsort_select_unstable");

#[inline]
pub fn sort<T>(v: &mut [T])
where
    T: Ord,
{
    if v.len() > 0 {
        partition_at_index(v, v.len() / 2, |a, b| a.lt(b));
    }
}

#[inline]
pub fn sort_by<T, F>(v: &mut [T], mut compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    if v.len() > 0 {
        partition_at_index(v, v.len() / 2, |a, b| compare(a, b).is_lt());
    }
}

/// Reorder the slice such that the element at `index` is at its final sorted position.
fn partition_at_index<T, F>(
    v: &mut [T],
    index: usize,
    mut is_less: F,
) -> (&mut [T], &mut T, &mut [T])
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    if index >= len {
        panic!(
            "partition_at_index index {} greater than length of slice {}",
            index, len
        );
    }

    if T::IS_ZST {
        // Sorting has no meaningful behavior on zero-sized types. Do nothing.
    } else if index == len - 1 {
        // Find max element and place it in the last position of the array. We're free to use
        // `unwrap()` here because we checked that `v` is not empty.
        let max_idx = max_index(v, &mut is_less).unwrap();
        v.swap(max_idx, index);
    } else if index == 0 {
        // Find min element and place it in the first position of the array. We're free to use
        // `unwrap()` here because we checked that `v` is not empty.
        let min_idx = min_index(v, &mut is_less).unwrap();
        v.swap(min_idx, index);
    } else {
        partition_at_index_loop(v, index, None, &mut is_less);
    }

    let (left, right) = v.split_at_mut(index);
    let (pivot, right) = right.split_at_mut(1);
    let pivot = &mut pivot[0];
    (left, pivot, right)
}

// For small sub-slices it's faster to use a dedicated small-sort, but because it is only called at
// most once, it doesn't make sense to use something more sophisticated than insertion-sort.
const INSERTION_SORT_THRESHOLD: usize = 16;

fn partition_at_index_loop<'a, T, F>(
    mut v: &'a mut [T],
    mut index: usize,
    mut ancestor_pivot: Option<&'a T>,
    is_less: &mut F,
) where
    F: FnMut(&T, &T) -> bool,
{
    // Limit the amount of iterations and fall back to fast deterministic selection
    // to ensure O(n) worst case running time. This limit needs to be constant, because
    // using `ilog2(len)` like in `sort` would result in O(n log n) time complexity.
    // The exact value of the limit is chosen somewhat arbitrarily, but for most inputs bad pivot
    // selections should be relatively rare, so the limit usually shouldn't be reached
    // anyways.
    let mut limit = 16;

    loop {
        if v.len() <= INSERTION_SORT_THRESHOLD {
            if v.len() > 1 {
                insertion_sort_shift_left(v, 1, is_less);
            }
            return;
        }

        if limit == 0 {
            median_of_medians(v, is_less, index);
            return;
        }

        limit -= 1;

        // Choose a pivot
        let pivot_pos = choose_pivot(v, is_less);

        // If the chosen pivot is equal to the predecessor, then it's the smallest element in the
        // slice. Partition the slice into elements equal to and elements greater than the pivot.
        // This case is usually hit when the slice contains many duplicate elements.
        if let Some(p) = ancestor_pivot {
            if !is_less(p, unsafe { v.get_unchecked(pivot_pos) }) {
                let num_lt = partition(v, pivot_pos, &mut |a, b| !is_less(b, a));

                // Continue sorting elements greater than the pivot. We know that `mid` contains
                // the pivot. So we can continue after `mid`.
                let mid = num_lt + 1;

                // If we've passed our index, then we're good.
                if mid > index {
                    return;
                }

                v = &mut v[mid..];
                index = index - mid;
                ancestor_pivot = None;
                continue;
            }
        }

        let mid = partition(v, pivot_pos, is_less);

        // Split the slice into `left`, `pivot`, and `right`.
        let (left, right) = v.split_at_mut(mid);
        let (pivot, right) = right.split_at_mut(1);
        let pivot = &pivot[0];

        if mid < index {
            v = right;
            index = index - mid - 1;
            ancestor_pivot = Some(pivot);
        } else if mid > index {
            v = left;
        } else {
            // If mid == index, then we're done, since partition() guaranteed that all elements
            // after mid are greater than or equal to mid.
            return;
        }
    }
}

/// Helper function that returns the index of the minimum element in the slice using the given
/// comparator function
fn min_index<T, F: FnMut(&T, &T) -> bool>(slice: &[T], is_less: &mut F) -> Option<usize> {
    slice
        .iter()
        .enumerate()
        .reduce(|acc, t| if is_less(t.1, acc.1) { t } else { acc })
        .map(|(i, _)| i)
}

/// Helper function that returns the index of the maximum element in the slice using the given
/// comparator function
fn max_index<T, F: FnMut(&T, &T) -> bool>(slice: &[T], is_less: &mut F) -> Option<usize> {
    slice
        .iter()
        .enumerate()
        .reduce(|acc, t| if is_less(acc.1, t.1) { t } else { acc })
        .map(|(i, _)| i)
}

/// Selection algorithm to select the k-th element from the slice in guaranteed O(n) time.
/// This is essentially a quickselect that uses Tukey's Ninther for pivot selection
fn median_of_medians<T, F: FnMut(&T, &T) -> bool>(mut v: &mut [T], is_less: &mut F, mut k: usize) {
    // Since this function isn't public, it should never be called with an out-of-bounds index.
    debug_assert!(k < v.len());

    // If T is as ZST, `partition_at_index` will already return early.
    debug_assert!(!T::IS_ZST);

    // We now know that `k < v.len() <= isize::MAX`
    loop {
        if v.len() <= INSERTION_SORT_THRESHOLD {
            if v.len() > 1 {
                insertion_sort_shift_left(v, 1, is_less);
            }
            return;
        }

        // `median_of_{minima,maxima}` can't handle the extreme cases of the first/last element,
        // so we catch them here and just do a linear search.
        if k == v.len() - 1 {
            // Find max element and place it in the last position of the array. We're free to use
            // `unwrap()` here because we know v must not be empty.
            let max_idx = max_index(v, is_less).unwrap();
            v.swap(max_idx, k);
            return;
        } else if k == 0 {
            // Find min element and place it in the first position of the array. We're free to use
            // `unwrap()` here because we know v must not be empty.
            let min_idx = min_index(v, is_less).unwrap();
            v.swap(min_idx, k);
            return;
        }

        let p = median_of_ninthers(v, is_less);

        if p == k {
            return;
        } else if p > k {
            v = &mut v[..p];
        } else {
            // Since `p < k < v.len()`, `p + 1` doesn't overflow and is
            // a valid index into the slice.
            v = &mut v[p + 1..];
            k -= p + 1;
        }
    }
}

// Optimized for when `k` lies somewhere in the middle of the slice. Selects a pivot
// as close as possible to the median of the slice. For more details on how the algorithm
// operates, refer to the paper <https://drops.dagstuhl.de/opus/volltexte/2017/7612/pdf/LIPIcs-SEA-2017-24.pdf>.
fn median_of_ninthers<T, F: FnMut(&T, &T) -> bool>(v: &mut [T], is_less: &mut F) -> usize {
    // use `saturating_mul` so the multiplication doesn't overflow on 16-bit platforms.
    let frac = if v.len() <= 1024 {
        v.len() / 12
    } else if v.len() <= 128_usize.saturating_mul(1024) {
        v.len() / 64
    } else {
        v.len() / 1024
    };

    let pivot = frac / 2;
    let lo = v.len() / 2 - pivot;
    let hi = frac + lo;
    let gap = (v.len() - 9 * frac) / 4;
    let mut a = lo - 4 * frac - gap;
    let mut b = hi + gap;
    for i in lo..hi {
        ninther(
            v,
            is_less,
            a,
            i - frac,
            b,
            a + 1,
            i,
            b + 1,
            a + 2,
            i + frac,
            b + 2,
        );
        a += 3;
        b += 3;
    }

    median_of_medians(&mut v[lo..lo + frac], is_less, pivot);
    partition(v, lo + pivot, is_less)
}

/// Moves around the 9 elements at the indices a..i, such that
/// `v[d]` contains the median of the 9 elements and the other
/// elements are partitioned around it.
fn ninther<T, F: FnMut(&T, &T) -> bool>(
    v: &mut [T],
    is_less: &mut F,
    a: usize,
    mut b: usize,
    c: usize,
    mut d: usize,
    e: usize,
    mut f: usize,
    g: usize,
    mut h: usize,
    i: usize,
) {
    b = median_idx(v, is_less, a, b, c);
    h = median_idx(v, is_less, g, h, i);
    if is_less(&v[h], &v[b]) {
        mem::swap(&mut b, &mut h);
    }
    if is_less(&v[f], &v[d]) {
        mem::swap(&mut d, &mut f);
    }
    if is_less(&v[e], &v[d]) {
        // do nothing
    } else if is_less(&v[f], &v[e]) {
        d = f;
    } else {
        if is_less(&v[e], &v[b]) {
            v.swap(e, b);
        } else if is_less(&v[h], &v[e]) {
            v.swap(e, h);
        }
        return;
    }
    if is_less(&v[d], &v[b]) {
        d = b;
    } else if is_less(&v[h], &v[d]) {
        d = h;
    }

    v.swap(d, e);
}

/// returns the index pointing to the median of the 3
/// elements `v[a]`, `v[b]` and `v[c]`
fn median_idx<T, F: FnMut(&T, &T) -> bool>(
    v: &[T],
    is_less: &mut F,
    mut a: usize,
    b: usize,
    mut c: usize,
) -> usize {
    if is_less(&v[c], &v[a]) {
        mem::swap(&mut a, &mut c);
    }
    if is_less(&v[c], &v[b]) {
        return c;
    }
    if is_less(&v[b], &v[a]) {
        return a;
    }
    b
}

struct CopyOnDrop<T> {
    src: *const T,
    dst: *mut T,
    len: usize,
}

impl<T> Drop for CopyOnDrop<T> {
    fn drop(&mut self) {
        // SAFETY: `src` must contain `len` initialized elements, and dst must
        // be valid to write `len` elements.
        unsafe {
            ptr::copy_nonoverlapping(self.src, self.dst, self.len);
        }
    }
}

/// Sorts range [begin, tail] assuming [begin, tail) is already sorted.
///
/// # Safety
/// begin < tail and p must be valid and initialized for all begin <= p <= tail.
unsafe fn insert_tail<T, F: FnMut(&T, &T) -> bool>(begin: *mut T, tail: *mut T, is_less: &mut F) {
    // SAFETY: see individual comments.
    unsafe {
        // SAFETY: in-bounds as tail > begin.
        let mut sift = tail.sub(1);
        if !is_less(&*tail, &*sift) {
            return;
        }

        // SAFETY: after this read tail is never read from again, as we only ever
        // read from sift, sift < tail and we only ever decrease sift. Thus this is
        // effectively a move, not a copy. Should a panic occur, or we have found
        // the correct insertion position, gap_guard ensures the element is moved
        // back into the array.
        let tmp = ManuallyDrop::new(tail.read());
        let mut gap_guard = CopyOnDrop {
            src: &*tmp,
            dst: tail,
            len: 1,
        };

        loop {
            // SAFETY: we move sift into the gap (which is valid), and point the
            // gap guard destination at sift, ensuring that if a panic occurs the
            // gap is once again filled.
            ptr::copy_nonoverlapping(sift, gap_guard.dst, 1);
            gap_guard.dst = sift;

            if sift == begin {
                break;
            }

            // SAFETY: we checked that sift != begin, thus this is in-bounds.
            sift = sift.sub(1);
            if !is_less(&tmp, &*sift) {
                break;
            }
        }
    }
}

/// Sort `v` assuming `v[..offset]` is already sorted.
pub fn insertion_sort_shift_left<T, F: FnMut(&T, &T) -> bool>(
    v: &mut [T],
    offset: usize,
    is_less: &mut F,
) {
    let len = v.len();
    if offset == 0 || offset > len {
        intrinsics::abort();
    }

    // SAFETY: see individual comments.
    unsafe {
        // We write this basic loop directly using pointers, as when we use a
        // for loop LLVM likes to unroll this loop which we do not want.
        // SAFETY: v_end is the one-past-end pointer, and we checked that
        // offset <= len, thus tail is also in-bounds.
        let v_base = v.as_mut_ptr();
        let v_end = v_base.add(len);
        let mut tail = v_base.add(offset);
        while tail != v_end {
            // SAFETY: v_base and tail are both valid pointers to elements, and
            // v_base < tail since we checked offset != 0.
            insert_tail(v_base, tail, is_less);

            // SAFETY: we checked that tail is not yet the one-past-end pointer.
            tail = tail.add(1);
        }
    }
}

// Recursively select a pseudomedian if above this threshold.
const PSEUDO_MEDIAN_REC_THRESHOLD: usize = 64;

/// Selects a pivot from `v`. Algorithm taken from glidesort by Orson Peters.
///
/// This chooses a pivot by sampling an adaptive amount of points, approximating
/// the quality of a median of sqrt(n) elements.
pub fn choose_pivot<T, F: FnMut(&T, &T) -> bool>(v: &[T], is_less: &mut F) -> usize {
    // We use unsafe code and raw pointers here because we're dealing with
    // heavy recursion. Passing safe slices around would involve a lot of
    // branches and function call overhead.

    let len = v.len();
    if len < 8 {
        intrinsics::abort();
    }

    // SAFETY: a, b, c point to initialized regions of len_div_8 elements,
    // satisfying median3 and median3_rec's preconditions as v_base points
    // to an initialized region of n = len elements.
    unsafe {
        let v_base = v.as_ptr();
        let len_div_8 = len / 8;

        let a = v_base; // [0, floor(n/8))
        let b = v_base.add(len_div_8 * 4); // [4*floor(n/8), 5*floor(n/8))
        let c = v_base.add(len_div_8 * 7); // [7*floor(n/8), 8*floor(n/8))

        if len < PSEUDO_MEDIAN_REC_THRESHOLD {
            median3(&*a, &*b, &*c, is_less).offset_from_unsigned(v_base)
        } else {
            median3_rec(a, b, c, len_div_8, is_less).offset_from_unsigned(v_base)
        }
    }
}

/// Calculates an approximate median of 3 elements from sections a, b, c, or
/// recursively from an approximation of each, if they're large enough. By
/// dividing the size of each section by 8 when recursing we have logarithmic
/// recursion depth and overall sample from f(n) = 3*f(n/8) -> f(n) =
/// O(n^(log(3)/log(8))) ~= O(n^0.528) elements.
///
/// SAFETY: a, b, c must point to the start of initialized regions of memory of
/// at least n elements.
unsafe fn median3_rec<T, F: FnMut(&T, &T) -> bool>(
    mut a: *const T,
    mut b: *const T,
    mut c: *const T,
    n: usize,
    is_less: &mut F,
) -> *const T {
    // SAFETY: a, b, c still point to initialized regions of n / 8 elements,
    // by the exact same logic as in choose_pivot.
    unsafe {
        if n * 8 >= PSEUDO_MEDIAN_REC_THRESHOLD {
            let n8 = n / 8;
            a = median3_rec(a, a.add(n8 * 4), a.add(n8 * 7), n8, is_less);
            b = median3_rec(b, b.add(n8 * 4), b.add(n8 * 7), n8, is_less);
            c = median3_rec(c, c.add(n8 * 4), c.add(n8 * 7), n8, is_less);
        }
        median3(&*a, &*b, &*c, is_less)
    }
}

/// Calculates the median of 3 elements.
///
/// SAFETY: a, b, c must be valid initialized elements.
#[inline(always)]
fn median3<T, F: FnMut(&T, &T) -> bool>(a: &T, b: &T, c: &T, is_less: &mut F) -> *const T {
    // Compiler tends to make this branchless when sensible, and avoids the
    // third comparison when not.
    let x = is_less(a, b);
    let y = is_less(a, c);
    if x == y {
        // If x=y=0 then b, c <= a. In this case we want to return max(b, c).
        // If x=y=1 then a < b, c. In this case we want to return min(b, c).
        // By toggling the outcome of b < c using XOR x we get this behavior.
        let z = is_less(b, c);
        if z ^ x {
            c
        } else {
            b
        }
    } else {
        // Either c <= a < b or b <= a < c, thus a is our median.
        a
    }
}

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

    // This construct is used to limit the LLVM IR generated, which saves large amounts of
    // compile-time by only instantiating the code that is needed. Idea by Frank Steffahn.
    let num_lt = (const { inst_partition::<T, F>() })(v_without_pivot, pivot, is_less);

    // Place the pivot between the two partitions.
    v.swap(0, num_lt);

    num_lt
}

const fn inst_partition<T, F: FnMut(&T, &T) -> bool>() -> fn(&mut [T], &T, &mut F) -> usize {
    const MAX_BRANCHLESS_PARTITION_SIZE: usize = 96;
    if mem::size_of::<T>() <= MAX_BRANCHLESS_PARTITION_SIZE {
        // Specialize for types that are relatively cheap to copy, where branchless optimizations
        // have large leverage e.g. `u64` and `String`.
        partition_lomuto_branchless_cyclic::<T, F>
    } else {
        partition_hoare_branchy_cyclic::<T, F>
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

        left.offset_from_unsigned(v_base)

        // `gap_opt` goes out of scope and overwrites the last wrong-side element on the right side
        // with the first wrong-side element of the left side that was initially overwritten by the
        // first wrong-side element on the right side element.
    }
}

struct PartitionState<T> {
    // The current element that is being looked at, scans left to right through slice.
    right: *mut T,
    // Counts the number of elements that compared less-than, also works around:
    // https://github.com/rust-lang/rust/issues/117128
    num_lt: usize,
    // Gap guard that tracks the temporary duplicate in the input.
    gap: GapGuardRaw<T>,
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
        let mut loop_body = |state: &mut PartitionState<T>| {
            let right_is_lt = is_less(&*state.right, pivot);
            let left = v_base.add(state.num_lt);

            ptr::copy(left, state.gap.pos, 1);
            ptr::copy_nonoverlapping(state.right, left, 1);

            state.gap.pos = state.right;
            state.num_lt += right_is_lt as usize;

            state.right = state.right.add(1);
        };

        // Ideally we could just use GapGuard in PartitionState, but the reference that is
        // materialized with `&mut state` when calling `loop_body` would create a mutable reference
        // to the parent struct that contains the gap value, invalidating the reference pointer
        // created from a reference to the gap value in the cleanup loop. This is only an issue
        // under Stacked Borrows, Tree Borrows accepts the intuitive code using GapGuard as valid.
        let mut gap_value = ManuallyDrop::new(ptr::read(v_base));

        let mut state = PartitionState {
            num_lt: 0,
            right: v_base.add(1),

            gap: GapGuardRaw {
                pos: v_base,
                value: &mut *gap_value,
            },
        };

        // Manual unrolling that works well on x86, Arm and with opt-level=s without murdering
        // compile-times. Leaving this to the compiler yields ok to bad results.
        let unroll_len = if const { mem::size_of::<T>() <= 16 } {
            2
        } else {
            1
        };

        let unroll_end = v_base.add(len - (unroll_len - 1));
        while state.right < unroll_end {
            if unroll_len == 2 {
                loop_body(&mut state);
                loop_body(&mut state);
            } else {
                loop_body(&mut state);
            }
        }

        // Single instantiate `loop_body` for both the unroll cleanup and cyclic permutation
        // cleanup. Optimizes binary-size and compile-time.
        let end = v_base.add(len);
        loop {
            let is_done = state.right == end;
            state.right = if is_done {
                state.gap.value
            } else {
                state.right
            };

            loop_body(&mut state);

            if is_done {
                mem::forget(state.gap);
                break;
            }
        }

        state.num_lt
    }
}

struct GapGuard<T> {
    pos: *mut T,
    value: ManuallyDrop<T>,
}

impl<T> Drop for GapGuard<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::copy_nonoverlapping(&*self.value, self.pos, 1);
        }
    }
}

/// Ideally this wouldn't be needed and we could just use the regular GapGuard.
/// See comment in [`partition_lomuto_branchless_cyclic`].
struct GapGuardRaw<T> {
    pos: *mut T,
    value: *mut T,
}

impl<T> Drop for GapGuardRaw<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::copy_nonoverlapping(self.value, self.pos, 1);
        }
    }
}
