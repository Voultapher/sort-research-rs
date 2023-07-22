//! A Kind of branchless Lomuto partition paired with a cyclic permutation.
//! TODO explain properly why this is good.

use core::mem::ManuallyDrop;
use core::ptr;

partition_impl!("scan_branchless_cyclic");

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
        // TODO try lt_count direct init here.
        let tmp_is_lt = is_less(&*arr_ptr, pivot);
        // TODO explain why doing this after the is_less call is important for non Freeze types.
        // TODO panic guard for tmp.
        let tmp = ManuallyDrop::new(ptr::read(arr_ptr));

        let mut tmp_val_drop_guard = SingleValueDropGuard {
            src: &*tmp,
            dest: arr_ptr,
        };

        macro_rules! left {
            () => {
                tmp_val_drop_guard.dest
            };
        }

        for i in 1..len {
            let right = arr_ptr.add(i);
            let right_is_lt = is_less(&*right, pivot);

            ptr::copy(right, left!(), 1);

            left!() = left!().add(right_is_lt as usize);

            let new_left_dest = if right_is_lt { right } else { left!() };
            ptr::copy(left!(), new_left_dest, 1);
        }

        let lt_count = left!().sub_ptr(arr_ptr) + (tmp_is_lt as usize);

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
