use core::mem;
use core::ptr;

partition_impl!("small_fast");

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    if len > SWAP {
        debug_assert!(false);
        return 0;
    }

    const SWAP: usize = 128;

    let arr_ptr = v.as_mut_ptr();

    let mut swap = mem::MaybeUninit::<[T; SWAP]>::uninit();
    let mut swap_ptr_l = swap.as_mut_ptr() as *mut T;
    let mut swap_ptr_r = unsafe { swap_ptr_l.add(len.saturating_sub(1)) };

    for i in 0..len {
        unsafe {
            let elem_ptr = arr_ptr.add(i);

            let is_l = is_less(&*elem_ptr, pivot);

            ptr::copy_nonoverlapping(elem_ptr, swap_ptr_l, 1);
            ptr::copy_nonoverlapping(elem_ptr, swap_ptr_r, 1);

            swap_ptr_l = swap_ptr_l.add(is_l as usize);
            swap_ptr_r = swap_ptr_r.sub(!is_l as usize);
        }
    }

    // SAFETY: swap now contains all elements that belong on the left side of the pivot. All
    // comparisons have been done if is_less would have panicked v would have stayed untouched.
    unsafe {
        let l_elems = swap_ptr_l.sub_ptr(swap.as_ptr() as *const T);

        // Now that swap has the correct order overwrite arr_ptr.
        ptr::copy_nonoverlapping(swap.as_ptr() as *const T, arr_ptr, len);

        l_elems
    }
}
