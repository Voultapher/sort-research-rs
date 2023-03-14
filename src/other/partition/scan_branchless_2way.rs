use core::mem::MaybeUninit;
use core::ptr;

partition_impl!("scan_branchless_2way");

/// Swap two values in array pointed to by a_ptr and b_ptr if b is less than a.
#[inline(always)]
pub unsafe fn branchless_swap_overlapping<T>(a_ptr: *mut T, b_ptr: *mut T, should_swap: bool) {
    // SAFETY: the caller must guarantee that `a_ptr` and `b_ptr` are valid for writes
    // and properly aligned, and part of the same allocation.

    // This is a branchless version of swap if.
    // The equivalent code with a branch would be:
    //
    // if should_swap {
    //     ptr::swap(a_ptr, b_ptr);
    // }

    // Give ourselves some scratch space to work with.
    // We do not have to worry about drops: `MaybeUninit` does nothing when dropped.
    let mut tmp = MaybeUninit::<T>::uninit();

    // The goal is to generate cmov instructions here.
    let a_swap_ptr = if should_swap { b_ptr } else { a_ptr };
    let b_swap_ptr = if should_swap { a_ptr } else { b_ptr };

    ptr::copy(b_swap_ptr, tmp.as_mut_ptr(), 1);
    ptr::copy(a_swap_ptr, a_ptr, 1);
    ptr::copy(tmp.as_ptr(), b_ptr, 1);
}

fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    let arr_ptr = v.as_mut_ptr();

    const UNROLL_SIZE: usize = 2;
    // assert!(UNROLL_SIZE.is_power_of_two());

    let len_mod = len % UNROLL_SIZE;
    let even_len = len - (len_mod != 0) as usize;
    let len_div_n = even_len / UNROLL_SIZE;

    unsafe {
        let mut fill_ptr_a = arr_ptr;
        let mut elem_ptr_a = fill_ptr_a; // TODO this can alias itself that is wrong.

        let mut fill_ptr_b = arr_ptr.add(len_div_n);
        let mut elem_ptr_b = fill_ptr_b;

        for _ in 0..len_div_n {
            let elem_is_less_a = is_less(&*elem_ptr_a, pivot);
            branchless_swap_overlapping(fill_ptr_a, elem_ptr_a, elem_is_less_a);
            fill_ptr_a = fill_ptr_a.add(elem_is_less_a as usize);
            elem_ptr_a = elem_ptr_a.add(1);

            let elem_is_less_b = is_less(&*elem_ptr_b, pivot);
            branchless_swap_overlapping(fill_ptr_b, elem_ptr_b, elem_is_less_b);
            fill_ptr_b = fill_ptr_b.add(elem_is_less_b as usize);
            elem_ptr_b = elem_ptr_b.add(1);
        }

        let is_less_count_a = fill_ptr_a.sub_ptr(arr_ptr);
        let is_less_count_b = fill_ptr_b.sub_ptr(arr_ptr) - len_div_n;

        ptr::swap_nonoverlapping(
            arr_ptr.add(is_less_count_a),
            arr_ptr.add(len_div_n),
            is_less_count_b,
        );

        let mut fill_ptr = arr_ptr.add(is_less_count_a + is_less_count_b);
        for elem in &mut v[(len - len_mod)..] {
            let elem_is_less = is_less(elem, pivot);
            branchless_swap_overlapping(elem, fill_ptr, elem_is_less);
            fill_ptr = fill_ptr.add(elem_is_less as usize);
        }

        fill_ptr.sub_ptr(arr_ptr)
    }
}
