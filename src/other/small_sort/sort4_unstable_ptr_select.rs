use core::cmp::Ordering;
use core::mem::MaybeUninit;
use core::ptr;

sort_impl!("sor4_unstable_ptr_select");

#[inline(always)]
pub fn cmp_select<T, F>(a_ptr: *const T, b_ptr: *const T, is_less: &mut F) -> (*const T, *const T)
where
    F: FnMut(&T, &T) -> bool,
{
    // TODO document.

    // SAFETY: TODO
    unsafe {
        let should_swap = is_less(&*b_ptr, &*a_ptr);
        if should_swap {
            (b_ptr, a_ptr)
        } else {
            (a_ptr, b_ptr)
        }
    }
}

// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
// performance impact.
#[inline(never)]
unsafe fn sort4_optimal<T, F>(v: &[T], dest_ptr: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 4.
    assert!(v.len() == 4);

    let arr_ptr = v.as_ptr();

    // Optimal sorting network see:
    // https://bertdobbelaere.github.io/sorting_networks.html.

    // SAFETY: We checked the len.
    unsafe {
        let (min_01_ptr, max_01_ptr) = cmp_select(arr_ptr.add(0), arr_ptr.add(1), is_less);
        let (min_23_ptr, max_23_ptr) = cmp_select(arr_ptr.add(2), arr_ptr.add(3), is_less);

        // Taking the min of the previous two smaller elements yields the global minimum.
        // We know that unknown_ptr_a is smaller than v[3] and larger or equal to min_ptr.
        let (min_ptr, unknown_ptr_a) = cmp_select(min_01_ptr, min_23_ptr, is_less);
        let (unknown_ptr_b, max_ptr) = cmp_select(max_01_ptr, max_23_ptr, is_less);

        let (res_1_ptr, res_2_ptr) = cmp_select(unknown_ptr_a, unknown_ptr_b, is_less);

        ptr::copy_nonoverlapping(min_ptr, dest_ptr.add(0), 1);
        ptr::copy_nonoverlapping(res_1_ptr, dest_ptr.add(1), 1);
        ptr::copy_nonoverlapping(res_2_ptr, dest_ptr.add(2), 1);
        ptr::copy_nonoverlapping(max_ptr, dest_ptr.add(3), 1);
    }
}

fn sort_impl<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let mut scratch = MaybeUninit::<[T; 4]>::uninit();
    let scratch_ptr = scratch.as_mut_ptr() as *mut T;

    unsafe {
        sort4_optimal(v, scratch_ptr, is_less);
        ptr::copy_nonoverlapping(scratch_ptr, v.as_mut_ptr(), 4);
    }
}

fn sort<T: Ord>(v: &mut [T]) {
    sort_impl(v, &mut |a, b| a.lt(b));
}

fn sort_by<T, F>(v: &mut [T], mut compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    sort_impl(v, &mut |a, b| compare(a, b) == Ordering::Less);
}
