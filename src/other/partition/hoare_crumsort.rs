use core::mem::MaybeUninit;
use core::ptr;

partition_impl!("hoare_crumsort");

struct FulcrumState<T> {
    r_ptr: *mut T,
    x_ptr: *mut T,
    elem_i: usize,
}

#[inline(always)]
unsafe fn fulcrum_rotate<T, F>(
    arr_ptr: *mut T,
    state: &mut FulcrumState<T>,
    offset_val: isize,
    loop_len: usize,
    pivot: &T,
    is_less: &mut F,
) where
    F: FnMut(&T, &T) -> bool,
{
    for _ in 0..loop_len {
        let is_l = is_less(&*state.x_ptr, pivot);
        let target_ptr = if is_l {
            arr_ptr.add(state.elem_i)
        } else {
            state.r_ptr.add(state.elem_i)
        };
        ptr::copy(state.x_ptr, target_ptr, 1);
        state.elem_i += is_l as usize;
        state.x_ptr = state.x_ptr.wrapping_offset(offset_val);
        state.r_ptr = state.r_ptr.wrapping_sub(1);
    }
}

unsafe fn small_aux_partition<T, F>(
    v: &mut [T],
    swap_ptr: *mut T,
    pivot: &T,
    is_less: &mut F,
) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    let arr_ptr = v.as_mut_ptr();

    // SAFETY: TODO
    unsafe {
        let mut swap_ptr_l = swap_ptr;
        let mut swap_ptr_r = swap_ptr.add(len - 1);

        // This could probably be sped-up by interleaving the two loops.
        for i in 0..len {
            let elem_ptr = arr_ptr.add(i);

            let is_l = is_less(&*elem_ptr, pivot);

            let target_ptr = if is_l { swap_ptr_l } else { swap_ptr_r };
            ptr::copy_nonoverlapping(elem_ptr, target_ptr, 1);

            swap_ptr_l = swap_ptr_l.add(is_l as usize);
            swap_ptr_r = swap_ptr_r.sub(!is_l as usize);
        }

        // SAFETY: swap now contains all elements that belong on the left side of the pivot. All
        // comparisons have been done if is_less would have panicked v would have stayed untouched.

        // Now that swap has the correct order overwrite arr_ptr.
        ptr::copy_nonoverlapping(swap_ptr, arr_ptr, len);

        swap_ptr_l.offset_from_unsigned(swap_ptr)
    }
}

// This function is *NOT* safe-to-use for non `Freeze` types.
fn partition<T, F: FnMut(&T, &T) -> bool>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize {
    // Novel partition implementation by Igor van den Hoven as part of his work in quadsort and
    // crumsort.

    let len = v.len();

    const ROTATION_ELEMS: usize = 32;

    let advance_left = |a_ptr: *const T, arr_ptr: *const T, elem_i: usize| -> bool {
        // SAFETY: TODO
        unsafe { (a_ptr.offset_from_unsigned(arr_ptr) - elem_i) <= ROTATION_ELEMS }
    };

    let mut swap = MaybeUninit::<[T; ROTATION_ELEMS * 2]>::uninit();
    let swap_ptr = swap.as_mut_ptr() as *mut T;

    let arr_ptr = v.as_mut_ptr();

    // SAFETY: TODO
    unsafe {
        if len <= (ROTATION_ELEMS * 2) {
            return small_aux_partition(v, swap_ptr, pivot, is_less);
        }
    }

    // SAFETY: TODO
    unsafe {
        ptr::copy_nonoverlapping(arr_ptr, swap_ptr, ROTATION_ELEMS);
        ptr::copy_nonoverlapping(
            arr_ptr.add(len - ROTATION_ELEMS),
            swap_ptr.add(ROTATION_ELEMS),
            ROTATION_ELEMS,
        );

        let mut state = FulcrumState {
            r_ptr: arr_ptr.add(len - 1),
            x_ptr: ptr::null_mut(),
            elem_i: 0,
        };

        let mut a_ptr = arr_ptr.add(ROTATION_ELEMS);
        let mut t_ptr = arr_ptr.add(len - (ROTATION_ELEMS + 1));

        for _ in 0..((len / ROTATION_ELEMS) - 2) {
            let loop_len = ROTATION_ELEMS;
            if advance_left(a_ptr, arr_ptr, state.elem_i) {
                state.x_ptr = a_ptr;
                fulcrum_rotate(arr_ptr, &mut state, 1, loop_len, pivot, is_less);
                a_ptr = state.x_ptr;
            } else {
                state.x_ptr = t_ptr;
                fulcrum_rotate(arr_ptr, &mut state, -1, loop_len, pivot, is_less);
                t_ptr = state.x_ptr;
            }
        }

        let loop_len = len % ROTATION_ELEMS;
        if advance_left(a_ptr, arr_ptr, state.elem_i) {
            state.x_ptr = a_ptr;
            fulcrum_rotate(arr_ptr, &mut state, 1, loop_len, pivot, is_less);
        } else {
            state.x_ptr = t_ptr;
            fulcrum_rotate(arr_ptr, &mut state, -1, loop_len, pivot, is_less);
        }

        let loop_len = ROTATION_ELEMS * 2;
        state.x_ptr = swap_ptr;
        fulcrum_rotate(arr_ptr, &mut state, 1, loop_len, pivot, is_less);

        state.elem_i
    }
}
