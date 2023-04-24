use core::cmp::Ordering;
use core::mem::MaybeUninit;
use core::ptr;

sort_impl!("sort10_unstable_experimental");

pub fn cmp_swap<T, F>(a_ptr: &mut *const T, b_ptr: &mut *const T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // TODO document.

    // SAFETY: TODO
    unsafe {
        let should_swap = is_less(&**b_ptr, &**a_ptr);

        let tmp = *a_ptr;
        *a_ptr = if should_swap { *b_ptr } else { *a_ptr };
        *b_ptr = if should_swap { tmp } else { *b_ptr };

        // // TODO explain.
        // if const { mem::size_of::<T>() <= mem::size_of::<usize>() } {
        //     *b_ptr = if should_swap { tmp } else { *b_ptr };
        // } else {
        //     *b_ptr = tmp.offset(b_ptr.offset_from(*a_ptr));
        // }
    }
}

// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
// performance impact.
#[inline(never)]
unsafe fn sort10_optimal<T, F>(v: &[T], dest_ptr: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 10.
    assert!(v.len() == 10);

    let arr_ptr = v.as_ptr();

    // Optimal sorting network see:
    // https://bertdobbelaere.github.io/sorting_networks.html.

    // We checked the len.
    unsafe {
        let mut val_0_ptr = arr_ptr.add(0);
        let mut val_1_ptr = arr_ptr.add(1);
        let mut val_2_ptr = arr_ptr.add(2);
        let mut val_3_ptr = arr_ptr.add(3);
        let mut val_4_ptr = arr_ptr.add(4);
        let mut val_5_ptr = arr_ptr.add(5);
        let mut val_6_ptr = arr_ptr.add(6);
        let mut val_7_ptr = arr_ptr.add(7);
        let mut val_8_ptr = arr_ptr.add(8);
        let mut val_9_ptr = arr_ptr.add(9);

        cmp_swap(&mut val_0_ptr, &mut val_8_ptr, is_less);
        cmp_swap(&mut val_1_ptr, &mut val_9_ptr, is_less);
        cmp_swap(&mut val_2_ptr, &mut val_7_ptr, is_less);
        cmp_swap(&mut val_3_ptr, &mut val_5_ptr, is_less);
        cmp_swap(&mut val_4_ptr, &mut val_6_ptr, is_less);
        cmp_swap(&mut val_0_ptr, &mut val_2_ptr, is_less);
        cmp_swap(&mut val_1_ptr, &mut val_4_ptr, is_less);
        cmp_swap(&mut val_5_ptr, &mut val_8_ptr, is_less);
        cmp_swap(&mut val_7_ptr, &mut val_9_ptr, is_less);
        cmp_swap(&mut val_0_ptr, &mut val_3_ptr, is_less);
        cmp_swap(&mut val_2_ptr, &mut val_4_ptr, is_less);
        cmp_swap(&mut val_5_ptr, &mut val_7_ptr, is_less);
        cmp_swap(&mut val_6_ptr, &mut val_9_ptr, is_less);
        cmp_swap(&mut val_0_ptr, &mut val_1_ptr, is_less);
        cmp_swap(&mut val_3_ptr, &mut val_6_ptr, is_less);
        cmp_swap(&mut val_8_ptr, &mut val_9_ptr, is_less);
        cmp_swap(&mut val_1_ptr, &mut val_5_ptr, is_less);
        cmp_swap(&mut val_2_ptr, &mut val_3_ptr, is_less);
        cmp_swap(&mut val_4_ptr, &mut val_8_ptr, is_less);
        cmp_swap(&mut val_6_ptr, &mut val_7_ptr, is_less);
        cmp_swap(&mut val_1_ptr, &mut val_2_ptr, is_less);
        cmp_swap(&mut val_3_ptr, &mut val_5_ptr, is_less);
        cmp_swap(&mut val_4_ptr, &mut val_6_ptr, is_less);
        cmp_swap(&mut val_7_ptr, &mut val_8_ptr, is_less);
        cmp_swap(&mut val_2_ptr, &mut val_3_ptr, is_less);
        cmp_swap(&mut val_4_ptr, &mut val_5_ptr, is_less);
        cmp_swap(&mut val_6_ptr, &mut val_7_ptr, is_less);
        cmp_swap(&mut val_3_ptr, &mut val_4_ptr, is_less);
        cmp_swap(&mut val_5_ptr, &mut val_6_ptr, is_less);

        ptr::copy_nonoverlapping(val_0_ptr, dest_ptr.add(0), 1);
        ptr::copy_nonoverlapping(val_1_ptr, dest_ptr.add(1), 1);
        ptr::copy_nonoverlapping(val_2_ptr, dest_ptr.add(2), 1);
        ptr::copy_nonoverlapping(val_3_ptr, dest_ptr.add(3), 1);
        ptr::copy_nonoverlapping(val_4_ptr, dest_ptr.add(4), 1);
        ptr::copy_nonoverlapping(val_5_ptr, dest_ptr.add(5), 1);
        ptr::copy_nonoverlapping(val_6_ptr, dest_ptr.add(6), 1);
        ptr::copy_nonoverlapping(val_7_ptr, dest_ptr.add(7), 1);
        ptr::copy_nonoverlapping(val_8_ptr, dest_ptr.add(8), 1);
        ptr::copy_nonoverlapping(val_9_ptr, dest_ptr.add(9), 1);
    }
}

fn sort_impl<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let mut scratch = MaybeUninit::<[T; 10]>::uninit();
    let scratch_ptr = scratch.as_mut_ptr() as *mut T;

    unsafe {
        sort10_optimal(v, scratch_ptr, is_less);

        if core::hint::black_box(cfg!(debug_assertions)) {
            ptr::copy_nonoverlapping(scratch_ptr, v.as_mut_ptr(), 10);
        }
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
