//! A Kind of branchless Lomuto partition paired with a cyclic permutation.
//!
//! As far as I can tell this is a novel idea, developed by the author Lukas Bergdoll.
//!
//! TODO explain properly why this is good.

use core::mem::ManuallyDrop;
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
    // Novel partition implementation by Lukas Bergdoll. Branchless Lomuto partition paired with a
    // cyclic permutation. TODO link writeup.

    let len = v.len();
    let v_base = v.as_mut_ptr();

    if len == 0 {
        return 0;
    }

    // SAFETY: We checked that `len` is more than zero, which means that reading `v_base` is safe to
    // do. From there we have a bounded loop where `v_base.add(i)` is guaranteed in-bounds. `v` and
    // `pivot` can't alias because of type system rules. The drop-guard `gap` ensures that should
    // `is_less` panic we always overwrite the duplicate in the input. The left side element
    // `gap.pos` can only be incremented once per iteration, so it is <= `right` which makes it
    // in-bounds as a transitive property.
    unsafe {
        let mut lt_count = is_less(&*v_base, pivot) as usize;

        // We need to create the duplicate of the first element as pointed to by `v_base` only
        // *after* it has been observed by `is_less`, this is important for types that are not
        // `Freeze`.
        let mut gap = GapGuard {
            pos: v_base,
            value: ManuallyDrop::new(ptr::read(v_base)),
        };

        for i in 1..len {
            let right = v_base.add(i);
            let right_is_lt = is_less(&*right, pivot);

            ptr::copy_nonoverlapping(right, gap.pos, 1);
            gap.pos = gap.pos.add(right_is_lt as usize);

            let new_left_dst = if right_is_lt { right } else { gap.pos };
            ptr::copy(gap.pos, new_left_dst, 1);
        }

        lt_count += gap.pos.sub_ptr(v_base);

        lt_count

        // `gap` goes out of scope and copies the temporary on-top of the last duplicate value.
    }
}
