//! The idea is to build a partition implementation for types u64 and smaller.

use std::ptr;

partition_impl!("lomuto_iterleaved");

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // TODO T: Freeze

    let len = v.len();
    let len_div_2 = len / 2;

    let arr_ptr = v.as_mut_ptr();

    // Always process two elements per loop iteration.
    // We compare the left and right element to pivot.
    //
    // lt == less than, ge == greater or equal
    //
    // Which gives us 4 scenarios:
    //
    // A) is_lt_left && is_lt_right
    // B) is_lt_left && is_ge_right
    // C) is_ge_left && is_lt_right
    // D) is_ge_left && is_ge_right
    //
    // A) swap left + 1 with right. New left = left + 2, new right = right.
    // B) swap left + 1 with right - 1. New left = left + 1, new right = right - 1.
    // C) swap left with right. New left = left + 1, new right = right - 1.
    // D) swap left with right - 1. new left = left, new right = right - 2.

    // SAFETY: TODO
    unsafe {
        let mut left = 0;
        let mut right = len - 1;

        // TODO uneven len.
        for _ in 0..len_div_2 {
            let is_lt_left = is_less(&*arr_ptr.add(left), pivot);
            let is_lt_right = is_less(&*arr_ptr.add(right), pivot);

            left += is_lt_left as usize;
            right -= !is_lt_right as usize;

            // TODO cyclic permutation.
            ptr::swap_nonoverlapping(arr_ptr.add(left), arr_ptr.add(right), 1);

            left += is_lt_right as usize;
            right -= !is_lt_left as usize;
        }

        left
    }
}

// Simple canonical implementation
// match (is_lt_left, is_lt_right) {
//     (true, true) => {
//         ptr::swap_nonoverlapping(left_ptr.add(1), right_ptr, 1);
//         left_ptr = left_ptr.add(2);
//     }
//     (true, false) => {
//         left_ptr = left_ptr.add(1);
//         right_ptr = right_ptr.sub(1);
//     }
//     (false, true) => {
//         ptr::swap_nonoverlapping(left_ptr, right_ptr, 1);
//         left_ptr = left_ptr.add(1);
//         right_ptr = right_ptr.sub(1);
//     }
//     (false, false) => {
//         ptr::swap_nonoverlapping(left_ptr, right_ptr.sub(1), 1);
//         right_ptr = right_ptr.sub(2);
//     }
// }
