use std::cmp::Ordering;
use std::mem;
use std::ptr;

mod median;
mod partition;

pub const FLUX_OUT: usize = 24;

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

mod std_lib {
    #![allow(unused_unsafe)]
    // This is taken from the stdlib.

    use super::*;

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
    fn shift_tail<T, F>(v: &mut [T], is_less: &mut F)
    where
        F: FnMut(&T, &T) -> bool,
    {
        let len = v.len();
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
            if len >= 2 && is_less(v.get_unchecked(len - 1), v.get_unchecked(len - 2)) {
                // Read the last element into a stack-allocated variable. If a following comparison
                // operation panics, `hole` will get dropped and automatically write the element back
                // into the slice.
                let tmp = mem::ManuallyDrop::new(ptr::read(v.get_unchecked(len - 1)));
                let v = v.as_mut_ptr();
                let mut hole = CopyOnDrop {
                    src: &*tmp,
                    dest: v.add(len - 2),
                };
                ptr::copy_nonoverlapping(v.add(len - 2), v.add(len - 1), 1);

                for i in (0..len - 2).rev() {
                    if !is_less(&*tmp, &*v.add(i)) {
                        break;
                    }

                    // Move `i`-th element one place to the right, thus shifting the hole to the left.
                    ptr::copy_nonoverlapping(v.add(i), v.add(i + 1), 1);
                    hole.dest = v.add(i);
                }
                // `hole` gets dropped and thus copies `tmp` into the remaining hole in `v`.
            }
        }
    }

    /// Sorts a slice using insertion sort, which is *O*(*n*^2) worst-case.
    pub fn insertion_sort<T, F>(v: &mut [T], is_less: &mut F)
    where
        F: FnMut(&T, &T) -> bool,
    {
        for i in 1..v.len() {
            shift_tail(&mut v[..i + 1], is_less);
        }
    }
}

pub enum Pattern {
    AlreadySorted,
    Reverse,
    None,
}

fn pattern_analyze<T, F>(arr: &[T], mut is_less: F) -> Pattern
where
    F: FnMut(&T, &T) -> bool,
{
    // The original analyzed for streaks, and while with enough trial and error a fast function can
    // be built. I'm not convinced that dispatching into a merge sort is worth the extra complexity
    // and cost. At least for now. TODO revisit.

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

#[inline]
#[allow(dead_code)]
fn flux_sort<T, F>(arr: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = arr.len();

    // Allocate a buffer to use as scratch memory. We keep the length 0 so we can keep in it
    // shallow copies of the contents of `v` without risking the dtors running on copies if
    // `is_less` panics.
    let mut swap = Vec::with_capacity(len);

    let arr_ptr = arr.as_mut_ptr();
    let swap_ptr = swap.as_mut_ptr();
    unsafe {
        partition::flux_partition(arr_ptr, swap_ptr, arr_ptr, swap_ptr.add(len), len, is_less);
    }
}

// #[inline]
// fn std_sort<T, F>(arr: &mut [T], is_less: &mut F)
// where
//     F: FnMut(&T, &T) -> bool,
// {
//     use std::cmp::Ordering;
//     arr.sort_by(|a, b| {
//         // This is a crime, should use proper underlying function.
//         if is_less(a, b) {
//             Ordering::Less
//         } else {
//             Ordering::Greater
//         }
//     });
// }

// Returns true if sorted false otherwise.
#[inline]
fn sort_small<T, F>(arr: &mut [T], mut is_less: F) -> bool
where
    F: FnMut(&T, &T) -> bool,
{
    if mem::size_of::<T>() == 0 {
        // Sorting has no meaningful behavior on zero-sized types. Do nothing.
        return true;
    }

    // Slices of up to this length get sorted using insertion sort.
    const MAX_INSERTION: usize = 20;

    let len = arr.len();

    match len {
        0 | 1 => (),
        2 => unsafe {
            sort2(arr, &mut is_less);
        },
        3 => unsafe {
            sort3(arr, &mut is_less);
        },
        4 => unsafe {
            sort4(arr, &mut is_less);
        },
        5..=8 => unsafe {
            sort4(arr, &mut is_less);
            insertion_sort_remaining(arr, 4, &mut is_less);
        },
        9..=MAX_INSERTION => {
            std_lib::insertion_sort(arr, &mut is_less);
        }
        _ => {
            let slice_bytes = len * mem::size_of::<T>();

            // For small slices that easily fit into L1 it's faster to analyze before sorting.
            // Even if that means walking through the array multiple times.
            if slice_bytes <= 2048 {
                match pattern_analyze(arr, &mut is_less) {
                    Pattern::AlreadySorted => (),
                    Pattern::Reverse => {
                        arr.reverse();
                    }
                    Pattern::None => {
                        return false;
                    }
                }
            } else {
                return false;
            }
        }
    }

    true
}

#[inline]
pub fn sort_by<T, F>(arr: &mut [T], mut compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    if !std::intrinsics::likely(sort_small(arr, |a, b| compare(a, b) == Ordering::Less)) {
        arr.sort_by(compare);
    }
}
