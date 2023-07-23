//! A Kind of branchless Lomuto partition paired with a cyclic permutation.
//!
//! As far as I can tell this is a novel idea, developed by the author Lukas Bergdoll.
//!
//! TODO explain properly why this is good.

use core::mem::ManuallyDrop;
use core::ptr;

partition_impl!("lomuto_branchless_cyclic");

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
        let tmp = ManuallyDrop::new(ptr::read(arr_ptr));

        let mut left = SingleValueDropGuard {
            src: &*tmp,
            dest: arr_ptr,
        };

        let end = arr_ptr.add(len - 1);
        let mut elem_ptr = arr_ptr.add(1);

        macro_rules! loop_body {
            () => {{
                let right = elem_ptr;
                let right_is_lt = is_less(&*right, pivot);

                ptr::copy_nonoverlapping(right, left.dest, 1);

                left.dest = left.dest.add(right_is_lt as usize);

                let new_left_dest = if right_is_lt { right } else { left.dest };
                ptr::copy(left.dest, new_left_dest, 1);

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

        lt_count += left.dest.sub_ptr(arr_ptr);

        lt_count

        // `tmp_val_drop_guard` goes out of scope and copies tmp on-top of the last duplicate value.
    }
}

struct SingleValueDropGuard<T> {
    src: *const T,
    dest: *mut T,
}

impl<T> Drop for SingleValueDropGuard<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::copy_nonoverlapping(self.src, self.dest, 1);
        }
    }
}
