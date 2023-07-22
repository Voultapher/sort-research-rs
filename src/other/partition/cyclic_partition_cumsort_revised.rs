use core::mem::MaybeUninit;
use core::ptr;

partition_impl!("cyclic_partition_cumsort_revised");

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

// Inspired by Igor van den Hoven and his work in quadsort/crumsort.
// TODO document.
fn fulcrum_partition_impl<T, F, const ROTATION_ELEMS: usize>(
    v: &mut [T],
    pivot: &T,
    is_less: &mut F,
) -> usize
where
    // T: Freeze,
    F: FnMut(&T, &T) -> bool,
{
    // TODO explain ideas. and panic safety. cleanup.
    let len = v.len();

    const SWAP_SIZE: usize = 64;

    assert!(len >= (ROTATION_ELEMS * 2) && ROTATION_ELEMS <= 32);

    let advance_left = |a_ptr: *const T, arr_ptr: *const T, elem_i: usize| -> bool {
        // SAFETY: TODO
        unsafe { (a_ptr.sub_ptr(arr_ptr) - elem_i) <= ROTATION_ELEMS }
    };

    let mut swap = MaybeUninit::<[T; SWAP_SIZE]>::uninit();
    let swap_ptr = swap.as_mut_ptr() as *mut T;

    let arr_ptr = v.as_mut_ptr();

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

fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    // T: Freeze,
    F: FnMut(&T, &T) -> bool,
{
    // TODO explain.
    if v.len() < 256 {
        fulcrum_partition_impl::<T, F, 16>(v, pivot, is_less)
    } else {
        fulcrum_partition_impl::<T, F, 32>(v, pivot, is_less)
    }
}
