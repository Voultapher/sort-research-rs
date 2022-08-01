#![allow(unused)]

use std::mem;
use std::ptr;

use rand::prelude::*;

use crate::fluxsort::swap_next_if_less;

unsafe fn median_of_three<T, F>(
    arr_ptr: *const T,
    v0: usize,
    v1: usize,
    v2: usize,
    is_less: &mut F,
) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: TODO

    let x = is_less(&*arr_ptr.add(v1), &*arr_ptr.add(v0)) as u8;
    let y = is_less(&*arr_ptr.add(v2), &*arr_ptr.add(v0)) as u8;
    let z = is_less(&*arr_ptr.add(v2), &*arr_ptr.add(v1)) as u8;

    let v = [v0, v1, v2];
    v[((x == y) as u8 + (y ^ z)) as usize]
}

pub unsafe fn median_of_nine<T, F>(
    arr_ptr: *const T,
    len: usize,
    is_less: &mut F,
) -> mem::MaybeUninit<T>
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: TODO

    let div = len / 16;

    let x = median_of_three(arr_ptr, div * 2, div * 1, div * 4, is_less);
    let y = median_of_three(arr_ptr, div * 8, div * 6, div * 10, is_less);
    let z = median_of_three(arr_ptr, div * 14, div * 12, div * 15, is_less);

    let pos = median_of_three(arr_ptr, x, y, z, is_less);

    // TODO panic safety for non Copy types.
    mem::MaybeUninit::new(arr_ptr.add(pos).read())
}

unsafe fn median_of_five<T, F>(
    arr_ptr: *const T,
    v0: usize,
    v1: usize,
    v2: usize,
    v3: usize,
    v4: usize,
    is_less: &mut F,
) -> mem::MaybeUninit<T>
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: TODO

    // TODO explain panic safety.
    let mut swap: [mem::MaybeUninit<T>; 4] = [
        mem::MaybeUninit::new(arr_ptr.add(v0).read()),
        mem::MaybeUninit::new(arr_ptr.add(v1).read()),
        mem::MaybeUninit::new(arr_ptr.add(v2).read()),
        mem::MaybeUninit::new(arr_ptr.add(v3).read()),
    ];

    let mut swap_ptr = swap.as_mut_ptr() as *mut T;

    swap_next_if_less(swap_ptr, is_less);
    swap_ptr = swap_ptr.add(2);

    swap_next_if_less(swap_ptr, is_less);
    swap_ptr = swap_ptr.offset(-2);

    // TODO swap_offset_if branchless like original, check perf.
    if is_less(&*swap_ptr.add(2), &*swap_ptr) {
        ptr::swap_nonoverlapping(swap_ptr, swap_ptr.add(2), 1);
    }
    swap_ptr = swap_ptr.add(1);

    if is_less(&*swap_ptr.add(2), &*swap_ptr) {
        ptr::swap_nonoverlapping(swap_ptr, swap_ptr.add(2), 1);
    }

    ptr::copy_nonoverlapping(arr_ptr.add(v4), swap_ptr.add(2), 1);

    let x = is_less(&*swap_ptr.add(1), &*swap_ptr) as u8;
    let y = is_less(&*swap_ptr.add(2), &*swap_ptr) as u8;
    let z = is_less(&*swap_ptr.add(2), &*swap_ptr.add(1)) as u8;

    mem::MaybeUninit::new(swap_ptr.add(((x == y) as u8 + (y ^ z)) as usize).read())
}

pub unsafe fn median_of_twentyfive<T, F>(
    arr_ptr: *const T,
    len: usize,
    is_less: &mut F,
) -> mem::MaybeUninit<T>
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: TODO
    debug_assert!(len > 2048);

    // TODO explain panic safety.
    let mut swap: [mem::MaybeUninit<T>; 5] = mem::MaybeUninit::uninit_array();
    let div = len / 64;

    let swap_ptr = swap.as_mut_ptr();

    swap_ptr.write(median_of_five(
        arr_ptr,
        div * 4,
        div * 1,
        div * 2,
        div * 8,
        div * 10,
        is_less,
    ));

    swap_ptr.add(1).write(median_of_five(
        arr_ptr,
        div * 16,
        div * 12,
        div * 14,
        div * 18,
        div * 20,
        is_less,
    ));

    swap_ptr.add(2).write(median_of_five(
        arr_ptr,
        div * 32,
        div * 24,
        div * 30,
        div * 34,
        div * 38,
        is_less,
    ));

    swap_ptr.add(3).write(median_of_five(
        arr_ptr,
        div * 48,
        div * 42,
        div * 44,
        div * 50,
        div * 52,
        is_less,
    ));

    swap_ptr.add(4).write(median_of_five(
        arr_ptr,
        div * 60,
        div * 54,
        div * 56,
        div * 62,
        div * 63,
        is_less,
    ));

    // TODO safety we know they have all been initialized now.
    median_of_five(swap_ptr as *const T, 0, 1, 2, 3, 4, is_less)
}

// TODO why does this need write access?
pub unsafe fn median_of_sqrt<T, F>(
    arr_ptr: *mut T,
    swap_ptr: *mut T,
    x_ptr: *mut T,
    len: usize,
    is_less: &mut F,
) -> mem::MaybeUninit<T>
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: TODO
    debug_assert!(len > 65536);

    let sqrt = if len > 262144 { 256 } else { 128 };
    let div = len / sqrt;

    let rand_offset = thread_rng().gen::<usize>() % sqrt;
    let mut a_ptr = x_ptr.add(rand_offset);

    let s_ptr = if x_ptr == arr_ptr { swap_ptr } else { arr_ptr };

    for i in 0..sqrt {
        // TODO panic safety. and why overlapping is fine.
        ptr::copy_nonoverlapping(a_ptr, s_ptr.add(i), 1);
        a_ptr = a_ptr.add(div);
    }

    // TODO quadsort_swap
    // 	FUNC(quadsort_swap)(pts, pts + sqrt, sqrt, sqrt, cmp);
    todo!();

    // mem::MaybeUninit::new(s_ptr.add(sqrt / 2).read())
}

// void FUNC(quadsort_swap)(void *array, void *swap, size_t swap_size, size_t nmemb, CMPFUNC *cmp)
// {
// 	if (nmemb < 32)
// 	{
// 		FUNC(tail_swap)(array, nmemb, cmp);
// 	}
// 	else if (FUNC(quad_swap)(array, nmemb, cmp) == 0)
// 	{
// 		size_t block;

// 		block = FUNC(quad_merge)(array, swap, swap_size, nmemb, 32, cmp);

// 		FUNC(blit_merge)(array, swap, swap_size, nmemb, block, cmp);
// 	}
// }
