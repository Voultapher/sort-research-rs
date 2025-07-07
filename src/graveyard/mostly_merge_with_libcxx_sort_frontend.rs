#![allow(unused_unsafe)]

use std::cmp::Ordering;
use std::mem;
use std::ptr;

#[inline]
pub fn sort<T>(arr: &mut [T])
where
    T: Ord,
{
    stable_sort(arr, |a, b| a.lt(b));
}

#[inline]
pub fn sort_by<T, F>(arr: &mut [T], mut compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    stable_sort(arr, |a, b| compare(a, b) == Ordering::Less);
}

#[inline]
pub fn stable_sort<T, F>(arr: &mut [T], mut is_less: F)
where
    F: FnMut(&T, &T) -> bool,
{
    stable_sort_impl(arr, &mut is_less);
}

#[inline]
pub fn stable_sort_impl<T, F>(arr: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // Slices of up to this length get sorted using insertion sort.
    const MAX_INSERTION: usize = 20;

    // Some types are expensive to move because they have a larger stack size,
    // and sortX uses swap_if_less which copies the type quite a bit.
    // To avoid a regression for such types, directly dispatch into merge sort.
    const MAX_SWAP_BYTES: usize = 16;

    if mem::size_of::<T>() == 0 {
        // Sorting has no meaningful behavior on zero-sized types. Do nothing.
        return;
    }

    if mem::size_of::<T>() > MAX_SWAP_BYTES {
        merge_sort(arr, is_less);
        return;
    }

    let len = arr.len();

    match len {
        0 | 1 => (),
        2 => unsafe {
            sort2(arr, is_less);
        },
        3 => unsafe {
            sort3(arr, is_less);
        },
        4 => unsafe {
            sort4(arr, is_less);
        },
        5..=8 => unsafe {
            sort4(arr, is_less);
            insertion_sort_remaining(arr, 4, is_less);
        },
        9..=MAX_INSERTION => {
            insertion_sort(arr, is_less);
        }
        _ => {
            merge_sort(arr, is_less);

            // let slice_bytes = len * mem::size_of::<T>();

            // // For small slices that easily fit into L1 it's faster to analyze before sorting.
            // // Even if that means walking through the array multiple times.
            // // This assumes comparing is relatively cheap.
            // if slice_bytes <= 4096 {
            //     match pattern_analyze(arr, is_less) {
            //         Pattern::AlreadySorted => (),
            //         Pattern::Reverse => {
            //             arr.reverse();
            //         }
            //         Pattern::None => {
            //             merge_sort(arr, is_less);
            //         }
            //     }
            // } else {
            //     merge_sort(arr, is_less);
            // }
        }
    }
}

pub enum Pattern {
    AlreadySorted,
    Reverse,
    None,
}

fn pattern_analyze<T, F>(arr: &[T], is_less: &mut F) -> Pattern
where
    F: FnMut(&T, &T) -> bool,
{
    // This function doesn't make sense for smaller slices.
    debug_assert!(!arr.is_empty());

    let mut sorted = 0usize;

    let len = arr.len();

    for i in 0..len.saturating_sub(1) {
        // SAFETY: We know that i is at most len - 2.
        let a = unsafe { arr.get_unchecked(i) };
        let b = unsafe { arr.get_unchecked(i + 1) };

        // PANIC SAFETY: we only have read access to arr.
        sorted += is_less(a, b) as usize;
    }

    match sorted {
        0 => Pattern::Reverse,
        _ if sorted == len - 1 => Pattern::AlreadySorted,
        _ => Pattern::None,
    }
}

/// Swap value with next value in array pointed to by arr_ptr if should_swap is true.
#[inline]
unsafe fn swap_next_if<T>(arr_ptr: *mut T, should_swap: bool) {
    // This is a branchless version of swap if.
    // The equivalent code with a branch would be:
    //
    // if should_swap {
    //     ptr::swap_nonoverlapping(arr_ptr, arr_ptr.add(1), 1)
    // }
    //
    // Be mindful in your benchmarking that this only starts to outperform branching code if the
    // benchmark doesn't execute the same branches again and again.

    // Give ourselves some scratch space to work with.
    // We do not have to worry about drops: `MaybeUninit` does nothing when dropped.
    let mut tmp = mem::MaybeUninit::<T>::uninit();

    // Perform the conditional swap.
    // SAFETY: the caller must guarantee that `arr_ptr` and `arr_ptr.add(1)` are
    // valid for writes and properly aligned. `tmp` cannot be overlapping either `arr_ptr` or
    // `arr_ptr.add(1) because `tmp` was just allocated on the stack as a separate allocated object.
    // And `arr_ptr` and `arr_ptr.add(1)` can't overlap either.
    // However `arr_ptr` and `arr_ptr.add(should_swap as usize)` can point to the same memory if
    // should_swap is false.
    ptr::copy_nonoverlapping(arr_ptr.add(!should_swap as usize), tmp.as_mut_ptr(), 1);
    ptr::copy(arr_ptr.add(should_swap as usize), arr_ptr, 1);
    ptr::copy_nonoverlapping(tmp.as_ptr(), arr_ptr.add(1), 1);
}

/// Swap value with next value in array pointed to by arr_ptr if should_swap is true.
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
    //
    // Important to only swap if it is more and not if it is equal. is_less should return false for
    // equal, so we don't swap.
    let should_swap = is_less(&*arr_ptr.add(1), &*arr_ptr);
    swap_next_if(arr_ptr, should_swap);
}

/// Sort the first 2 elements of arr.
unsafe fn sort2<T, F>(arr: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure arr is at least len 2.
    debug_assert!(arr.len() >= 2);

    swap_next_if_less(arr.as_mut_ptr(), is_less);
}

/// Sort the first 3 elements of arr.
unsafe fn sort3<T, F>(arr: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure arr is at least len 3.
    debug_assert!(arr.len() >= 3);

    let arr_ptr = arr.as_mut_ptr();
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

/// Sort the first 4 elements of arr.
unsafe fn sort4<T, F>(arr: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure arr is at least len 4.
    debug_assert!(arr.len() >= 4);

    let arr_ptr = arr.as_mut_ptr();
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

/// Sort the remaining elements after offset in arr.
unsafe fn insertion_sort_remaining<T, F>(arr: &mut [T], offset: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let arr_ptr = arr.as_mut_ptr();
    let len = arr.len();

    let mut i = offset;

    // This implementation is extremely simple, and beats other more complex implementations for the
    // narrow use case it's intended for.
    while i < len {
        let mut j = i;

        loop {
            if j == 0 {
                break;
            }

            let x1 = arr_ptr.add(j - 1);
            let x2 = arr_ptr.add(j);

            // PANIC SAFETY: if is_less panics, no scratch memory was created and the slice should
            // still be in a well defined state, without duplicates.
            if !is_less(&*x2, &*x1) {
                break;
            }

            ptr::swap_nonoverlapping(x1, x2, 1);
            j -= 1;
        }

        i += 1;
    }
}

// --- insertion sort ---

/// When dropped, copies from `src` into `dest`.
struct CopyOnDrop<T> {
    src: *const T,
    dest: *mut T,
}

impl<T> Drop for CopyOnDrop<T> {
    fn drop(&mut self) {
        // SAFETY:  This is a helper class.
        //          Please refer to its usage for correctness.
        //          Namely, one must be sure that `src` and `dst` does not overlap as required by `ptr::copy_nonoverlapping`.
        unsafe {
            ptr::copy_nonoverlapping(self.src, self.dest, 1);
        }
    }
}

/// Shifts the last element to the left until it encounters a smaller or equal element.
fn shift_tail<T, F>(arr: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = arr.len();
    // SAFETY: The unsafe operations below involves indexing without a bound check (by offsetting a
    // pointer) and copying memory (`ptr::copy_nonoverlapping`).
    //
    // a. Indexing:
    //  1. We checked the size of the array to >= 2.
    //  2. All the indexing that we will do is always between `0 <= index < len-1` at most.
    //
    // b. Memory copying
    //  1. We are obtaining pointers to references which are guaranteed to be valid.
    //  2. They cannot overlap because we obtain pointers to difference indices of the slice.
    //     Namely, `i` and `i+1`.
    //  3. If the slice is properly aligned, the elements are properly aligned.
    //     It is the caller's responsibility to make sure the slice is properly aligned.
    //
    // See comments below for further detail.
    unsafe {
        // If the last two elements are out-of-order...
        if len >= 2 && is_less(arr.get_unchecked(len - 1), arr.get_unchecked(len - 2)) {
            // Read the last element into a stack-allocated variable. If a following comparison
            // operation panics, `hole` will get dropped and automatically write the element back
            // into the slice.
            let tmp = mem::ManuallyDrop::new(ptr::read(arr.get_unchecked(len - 1)));
            let arr = arr.as_mut_ptr();
            let mut hole = CopyOnDrop {
                src: &*tmp,
                dest: arr.add(len - 2),
            };
            ptr::copy_nonoverlapping(arr.add(len - 2), arr.add(len - 1), 1);

            for i in (0..len - 2).rev() {
                if !is_less(&*tmp, &*arr.add(i)) {
                    break;
                }

                // Move `i`-th element one place to the right, thus shifting the hole to the left.
                ptr::copy_nonoverlapping(arr.add(i), arr.add(i + 1), 1);
                hole.dest = arr.add(i);
            }
            // `hole` gets dropped and thus copies `tmp` into the remaining hole in `arr`.
        }
    }
}

/// Sorts a slice using insertion sort, which is *O*(*n*^2) worst-case.
fn insertion_sort<T, F>(arr: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    for i in 1..arr.len() {
        shift_tail(&mut arr[..i + 1], is_less);
    }
}

// --- merge sort

// This is taken from the stdlib stable sort.

/// This merge sort borrows some (but not all) ideas from TimSort, which is described in detail
/// [here](https://github.com/python/cpython/blob/main/Objects/listsort.txt).
///
/// The algorithm identifies strictly descending and non-descending subsequences, which are called
/// natural runs. There is a stack of pending runs yet to be merged. Each newly found run is pushed
/// onto the stack, and then some pairs of adjacent runs are merged until these two invariants are
/// satisfied:
///
/// 1. for every `i` in `1..runs.len()`: `runs[i - 1].len > runs[i].len`
/// 2. for every `i` in `2..runs.len()`: `runs[i - 2].len > runs[i - 1].len + runs[i].len`
///
/// The invariants ensure that the total running time is *O*(*n* \* log(*n*)) worst-case.
#[inline]
fn merge_sort<T, F>(arr: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // Slices of up to this length get sorted using insertion sort.
    const MAX_INSERTION: usize = 20;

    let len = arr.len();

    // Short arrays get sorted in-place via insertion sort to avoid allocations. This only happens
    // for larger types, that are sensitive to their cost of moving their stack bytes. Smaller types
    // get handled by sort_stable.
    if len <= MAX_INSERTION {
        if len >= 2 {
            insertion_sort(arr, is_less);
        }
        return;
    }

    // Very short runs are extended using insertion sort to span at least this many elements.
    // Benchmarks show this is better adjusted based on input length.
    // Longer inputs perform better with higher min run length.
    let min_run: usize = (((len as f64).log(2.3) * 1.5).round() as usize)
        .min(23)
        .max(10);

    // Allocate a buffer to use as scratch memory. We keep the length 0 so we can keep in it
    // shallow copies of the contents of `arr` without risking the dtors running on copies if
    // `is_less` panics. When merging two sorted runs, this buffer holds a copy of the shorter run,
    // which will always have length at most `len / 2`.
    let mut buf = Vec::with_capacity(len / 2);

    // In order to identify natural runs in `arr`, we traverse it backwards. That might seem like a
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
                if is_less(arr.get_unchecked(start + 1), arr.get_unchecked(start)) {
                    while start > 0
                        && is_less(arr.get_unchecked(start), arr.get_unchecked(start - 1))
                    {
                        start -= 1;
                    }
                    arr[start..end].reverse();
                } else {
                    while start > 0
                        && !is_less(arr.get_unchecked(start), arr.get_unchecked(start - 1))
                    {
                        start -= 1;
                    }
                }
            }
        }

        // Insert some more elements into the run if it's too short. Insertion sort is faster than
        // merge sort on short sequences, so this significantly improves performance.
        let start_found = start;
        let start_end_diff = end - start;

        if start_end_diff == len {
            // The full thing is already sorted.
            // Bail out here to avoid costly merge logic.
            return;
        }

        if start_end_diff < min_run && start_end_diff >= 2 {
            start = start.saturating_sub(min_run - start_end_diff);

            for i in (start..start_found).rev() {
                // We ensured that the slice length is always at lest 2 long.
                unsafe {
                    insert_head(&mut arr[i..end], is_less);
                }
            }
        }

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
                    &mut arr[left.start..right.start + right.len],
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
        start: usize,
        len: usize,
    }
}

/// Inserts `arr[0]` into pre-sorted sequence `arr[1..]` so that whole `arr[..]` becomes sorted.
///
/// This is the integral subroutine of insertion sort.
#[inline]
unsafe fn insert_head<T, F>(arr: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    debug_assert!(arr.len() >= 2);

    if arr.len() >= 2 && is_less(&arr[1], &arr[0]) {
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
            let tmp = mem::ManuallyDrop::new(ptr::read(&arr[0]));

            // Intermediate state of the insertion process is always tracked by `hole`, which
            // serves two purposes:
            // 1. Protects integrity of `arr` from panics in `is_less`.
            // 2. Fills the remaining hole in `arr` in the end.
            //
            // Panic safety:
            //
            // If `is_less` panics at any point during the process, `hole` will get dropped and
            // fill the hole in `arr` with `tmp`, thus ensuring that `arr` still holds every object it
            // initially held exactly once.
            let mut hole = InsertionHole {
                src: &*tmp,
                dest: &mut arr[1],
            };
            ptr::copy_nonoverlapping(&arr[1], &mut arr[0], 1);

            for i in 2..arr.len() {
                if !is_less(&arr[i], &*tmp) {
                    break;
                }
                ptr::copy_nonoverlapping(&arr[i], &mut arr[i - 1], 1);
                hole.dest = &mut arr[i];
            }
            // `hole` gets dropped and thus copies `tmp` into the remaining hole in `arr`.
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

/// Merges non-decreasing runs `arr[..mid]` and `arr[mid..]` using `buf` as temporary storage, and
/// stores the result into `arr[..]`.
///
/// # Safety
///
/// The two slices must be non-empty and `mid` must be in bounds. Buffer `buf` must be long enough
/// to hold a copy of the shorter slice. Also, `T` must not be a zero-sized type.
unsafe fn merge<T, F>(arr: &mut [T], mid: usize, buf: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = arr.len();
    let arr = arr.as_mut_ptr();
    let (v_mid, v_end) = unsafe { (arr.add(mid), arr.add(len)) };

    // The merge process first copies the shorter run into `buf`. Then it traces the newly copied
    // run and the longer run forwards (or backwards), comparing their next unconsumed elements and
    // copying the lesser (or greater) one into `arr`.
    //
    // As soon as the shorter run is fully consumed, the process is done. If the longer run gets
    // consumed first, then we must copy whatever is left of the shorter run into the remaining
    // hole in `arr`.
    //
    // Intermediate state of the process is always tracked by `hole`, which serves two purposes:
    // 1. Protects integrity of `arr` from panics in `is_less`.
    // 2. Fills the remaining hole in `arr` if the longer run gets consumed first.
    //
    // Panic safety:
    //
    // If `is_less` panics at any point during the process, `hole` will get dropped and fill the
    // hole in `arr` with the unconsumed range in `buf`, thus ensuring that `arr` still holds every
    // object it initially held exactly once.
    let mut hole;

    if mid <= len - mid {
        // The left run is shorter.
        unsafe {
            ptr::copy_nonoverlapping(arr, buf, mid);
            hole = MergeHole {
                start: buf,
                end: buf.add(mid),
                dest: arr,
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

        while arr < *left && buf < *right {
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
    // it will now be copied into the hole in `arr`.

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
