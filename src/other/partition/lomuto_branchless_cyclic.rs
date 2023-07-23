//! A Kind of branchless Lomuto partition paired with a cyclic permutation.
//!
//! As far as I can tell this is a novel idea, developed by the author Lukas Bergdoll.
//!
//! TODO explain properly why this is good.

use core::mem::ManuallyDrop;
use core::ptr;

partition_impl!("lomuto_branchless_cyclic");

struct GapGuardOverlapping<T> {
    pos: *mut T,
    value: ManuallyDrop<T>,
}

impl<T> Drop for GapGuardOverlapping<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::write(self.pos, ManuallyDrop::take(&mut self.value));
        }
    }
}

fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    let arr_ptr = v.as_mut_ptr();

    if len == 0 {
        return 0;
    }

    unsafe {
        let mut lt_count = is_less(&*arr_ptr, pivot) as usize;

        // TODO explain why doing this after the is_less call is important for non Freeze types.
        let mut gap = GapGuardOverlapping {
            pos: arr_ptr,
            value: ManuallyDrop::new(ptr::read(arr_ptr)),
        };

        let end = arr_ptr.add(len - 1);
        let mut elem_ptr = arr_ptr.add(1);

        macro_rules! loop_body {
            () => {{
                let right = elem_ptr;
                let right_is_lt = is_less(&*right, pivot);

                ptr::copy_nonoverlapping(right, gap.pos, 1);

                gap.pos = gap.pos.add(right_is_lt as usize);

                let new_left_dest = if right_is_lt { right } else { gap.pos };
                ptr::copy(gap.pos, new_left_dest, 1);

                elem_ptr = elem_ptr.add(1);
                _ = elem_ptr;
            }};
        }

        while elem_ptr < end {
            for _ in 0..2 {
                loop_body!();
            }
        }

        if elem_ptr < arr_ptr.add(len) {
            loop_body!();
        }

        lt_count += gap.pos.sub_ptr(arr_ptr);

        lt_count

        // `tmp_val_drop_guard` goes out of scope and copies tmp on-top of the last duplicate value.
    }
}
