use core::intrinsics;

const PSEUDO_MEDIAN_REC_THRESHOLD: usize = 64;

use crate::smallsort::SmallSortImpl;

/// Selects a pivot from left, right.
///
/// Idea taken from glidesort by Orson Peters.
///
/// This chooses a pivot by sampling an adaptive amount of points, mimicking the median quality of
/// median of square root.
pub(crate) fn choose_pivot<T, F>(v: &[T], is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    if len < T::MAX_SMALL_SORT_LEN {
        // It's a logic bug if this get's called on slice that would be small-sorted.
        intrinsics::abort();
    }

    let len_div_2 = len / 2;
    let arr_ptr = v.as_ptr();

    // SAFETY: Assuming that `max_len_small_sort::<T>()` is larger than 0 all pointer calculations
    // below yield valid in-bounds pointers.
    unsafe {
        let median_guess_ptr = if len < PSEUDO_MEDIAN_REC_THRESHOLD {
            median3(
                arr_ptr,
                arr_ptr.add(len_div_2),
                arr_ptr.add(len - 1),
                is_less,
            )
        } else {
            let len_div_8 = len / 8;
            let a = arr_ptr;
            let b = arr_ptr.add(len_div_8 * 4);
            let c = arr_ptr.add(len_div_8 * 7);

            median3_rec(a, b, c, len_div_8, is_less)
        };

        median_guess_ptr.sub_ptr(arr_ptr)
    }
}

/// Calculates an approximate median of 3 elements from sections a, b, c, or recursively from an
/// approximation of each, if they're large enough. By dividing the size of each section by 8 when
/// recursing we have logarithmic recursion depth and overall sample from
/// f(n) = 3*f(n/8) -> f(n) = O(n^(log(3)/log(8))) ~= O(n^0.528) elements.
///
/// SAFETY: a, b, c must point to the start of initialized regions of memory of
/// at least n elements.
#[inline(never)]
unsafe fn median3_rec<T, F>(
    mut a: *const T,
    mut b: *const T,
    mut c: *const T,
    n: usize,
    is_less: &mut F,
) -> *const T
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: TODO
    unsafe {
        if n * 8 >= PSEUDO_MEDIAN_REC_THRESHOLD {
            let n8 = n / 8;
            a = median3_rec(a, a.add(n8 * 4), a.add(n8 * 7), n8, is_less);
            b = median3_rec(b, b.add(n8 * 4), b.add(n8 * 7), n8, is_less);
            c = median3_rec(c, c.add(n8 * 4), c.add(n8 * 7), n8, is_less);
        }
        median3(a, b, c, is_less)
    }
}

/// Calculates the median of 3 elements.
///
/// SAFETY: a, b, c must be valid initialized elements.
unsafe fn median3<T, F>(a: *const T, b: *const T, c: *const T, is_less: &mut F) -> *const T
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: TODO
    //
    // Compiler tends to make this branchless when sensible, and avoids the
    // third comparison when not.
    unsafe {
        let x = is_less(&*a, &*b);
        let y = is_less(&*a, &*c);
        if x == y {
            // If x=y=0 then b, c <= a. In this case we want to return max(b, c).
            // If x=y=1 then a < b, c. In this case we want to return min(b, c).
            // By toggling the outcome of b < c using XOR x we get this behavior.
            let z = is_less(&*b, &*c);

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
}
