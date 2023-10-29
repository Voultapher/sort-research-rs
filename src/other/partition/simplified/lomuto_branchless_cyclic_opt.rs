//! A Kind of branchless Lomuto partition paired with a cyclic permutation.
//!
//! As far as I can tell this is a novel idea, developed by the author Lukas Bergdoll.
//!
//! TODO explain properly why this is good.

use core::mem::{self, ManuallyDrop};
use core::ptr;

partition_impl!("lomuto_branchless_cyclic");

struct GapGuard<T> {
    pos: *mut T,
    value: ManuallyDrop<T>,
}

impl<T> Drop for GapGuard<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::copy_nonoverlapping(&*self.value, self.pos, 1);
        }
    }
}

fn partition<T, F: FnMut(&T, &T) -> bool>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize {
    // Novel partition implementation by Lukas Bergdoll and Orson Peters. Branchless Lomuto
    // partition paired with a cyclic permutation. TODO link writeup.

    let len = v.len();
    if len == 0 {
        return 0;
    }

    // SAFETY: We checked that `len` is more than zero, which means that reading `v_base` is safe to
    // do. From there we have a bounded loop where `v_base.add(i)` is guaranteed in-bounds. `v` and
    // `pivot` can't alias because of type system rules. The drop-guard `gap` ensures that should
    // `is_less` panic we always overwrite the duplicate in the input. `gap.pos` stores the previous
    // value of `right` and starts at `v_base` and so it too is in-bounds. We never pass the saved
    // `gap.value` to `is_less` while it is inside the `GapGuard` thus any changes via interior
    // mutability will be observed.
    unsafe {
        let v_base = v.as_mut_ptr();
        let mut left = v_base;

        let mut gap = GapGuard {
            pos: v_base,
            value: ManuallyDrop::new(ptr::read(v_base)),
        };

        for i in 1..len {
            let right = v_base.add(i);
            let right_is_lt = is_less(&*right, pivot);

            ptr::copy(left, gap.pos, 1);
            ptr::copy_nonoverlapping(right, left, 1);

            gap.pos = right;
            left = left.add(right_is_lt as usize);
        }

        ptr::copy(left, gap.pos, 1);
        ptr::copy_nonoverlapping(&*gap.value, left, 1);
        mem::forget(gap);

        let gap_value_is_lt = is_less(&*left, pivot);
        left = left.add(gap_value_is_lt as usize);

        let lt_count = left.sub_ptr(v_base);
        lt_count
    }
}
