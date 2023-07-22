use core::mem;
use core::ptr;

partition_impl!("cyclic_partition_simple");

// Demonstrate ideas behind rotation based partitioning.

pub fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // TODO explain ideas. and panic safety. cleanup.
    let len = v.len();

    // SAFETY: TODO
    unsafe {
        let next_left = |mut l_ptr: *mut T, r_ptr: *mut T, is_less: &mut F| -> *mut T {
            while (l_ptr < r_ptr) && is_less(&*l_ptr, pivot) {
                l_ptr = l_ptr.add(1);
            }

            l_ptr
        };

        let next_right = |l_ptr: *mut T, mut r_ptr: *mut T, is_less: &mut F| -> *mut T {
            // Find next value on the right side that needs to go on the left side.
            while (l_ptr < r_ptr) && !is_less(&*r_ptr, pivot) {
                r_ptr = r_ptr.sub(1);
            }

            r_ptr
        };

        let arr_ptr = v.as_mut_ptr();

        let mut l_ptr = arr_ptr;
        let mut r_ptr = arr_ptr.add(len - 1);

        l_ptr = next_left(l_ptr, r_ptr, is_less);
        r_ptr = next_right(l_ptr, r_ptr, is_less);

        let tmp = ptr::read(l_ptr);
        ptr::copy_nonoverlapping(r_ptr, l_ptr, 1);

        let mut drop_guard = InsertionHole {
            src: &tmp,
            dest: r_ptr,
        };

        while l_ptr < r_ptr {
            l_ptr = next_left(l_ptr, r_ptr, is_less);

            // Copy left wrong side element into right side wrong side element.
            ptr::copy_nonoverlapping(l_ptr, r_ptr, 1);

            // The drop_guard also participates in the rotation logic. Only requiring one update per
            // loop. The two places that could panic are next_left and next_right, If either of them
            // panics, drop_guard.dest will hold a spot that contains a duplicate element. Which
            // will be overwritten with the temporary value.
            drop_guard.dest = l_ptr;

            r_ptr = next_right(l_ptr, r_ptr, is_less);
            // Copy right wrong side element into left side wrong side element.
            ptr::copy_nonoverlapping(r_ptr, l_ptr, 1);
        }

        ptr::copy_nonoverlapping(&tmp, r_ptr, 1);
        mem::forget(drop_guard);

        l_ptr.sub_ptr(arr_ptr)
    }
}

// When dropped, copies from `src` into `dest`.
struct InsertionHole<T> {
    src: *const T,
    dest: *mut T,
}

impl<T> Drop for InsertionHole<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::copy_nonoverlapping(self.src, self.dest, 1);
        }
    }
}
