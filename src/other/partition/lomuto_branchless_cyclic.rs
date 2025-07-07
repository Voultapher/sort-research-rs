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
    // Novel partition implementation by Lukas Bergdoll. Branchless Lomuto partition paired with a
    // cyclic permutation. TODO link writeup.

    let len = v.len();
    let v_base = v.as_mut_ptr();

    if len == 0 {
        return 0;
    }

    // Manually unrolled as micro-optimization as only x86 gets auto-unrolling but not Arm.
    let unroll_len = if const { mem::size_of::<T>() <= 16 } {
        2
    } else {
        1
    };

    // SAFETY: We checked that `len` is more than zero, which means that reading `v_base` is safe to
    // do. From there we have a bounded loop where `v_base.add(i)` is guaranteed in-bounds. `v` and
    // `pivot` can't alias because of type system rules. The drop-guard `gap` ensures that should
    // `is_less` panic we always overwrite the duplicate in the input. The left side element
    // `gap.pos` can only be incremented once per iteration, so it is <= `right` which makes it
    // in-bounds as a transitive property.
    unsafe {
        let mut lt_count = is_less(&*v_base, pivot) as usize;

        // TODO explain why doing this after the is_less call is important for non Freeze types.
        let mut gap = GapGuard {
            pos: v_base,
            value: ManuallyDrop::new(ptr::read(v_base)),
        };

        let mut right = v_base.add(1);

        macro_rules! loop_body {
            () => {{
                let right_is_lt = is_less(&*right, pivot);

                ptr::copy_nonoverlapping(right, gap.pos, 1);
                gap.pos = gap.pos.add(right_is_lt as usize);

                let new_left_dest = if right_is_lt { right } else { gap.pos };
                ptr::copy(gap.pos, new_left_dest, 1);

                right = right.add(1);
            }};
        }

        let unroll_end = v_base.add(len - (unroll_len - 1));
        while right < unroll_end {
            for _ in 0..unroll_len {
                loop_body!();
            }
        }

        while right < v_base.add(len) {
            loop_body!();
        }

        lt_count += gap.pos.offset_from_unsigned(v_base);

        lt_count

        // `gap` goes out of scope and copies tmp on-top of the last duplicate value.
    }
}
