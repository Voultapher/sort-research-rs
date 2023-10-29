//! Same idea as lomuto_branchless_cyclic but refined by Orson Peters to avoid the cmov.

use core::mem::{self, ManuallyDrop};
use core::ptr;

partition_impl!("lomuto_branchless_cyclic_opt");

struct GapGuard<T> {
    pos: *mut T,
    value: ManuallyDrop<T>,
}

impl<T> Drop for GapGuard<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::write(self.pos, ManuallyDrop::take(&mut self.value));
        }
    }
}

fn partition<T, F: FnMut(&T, &T) -> bool>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize {
    // Novel partition implementation by Lukas Bergdoll and Orson Peters. Branchless Lomuto
    // partition paired with a cyclic permutation. TODO link writeup.

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
    // `is_less` panic we always overwrite the duplicate in the input. `gap.pos` stores the previous
    // value of `right` and starts at `v_base` and so it too is in-bounds. Given `UNROLL_LEN == 2`
    // after the main loop we either have A) the last element in `v` that has not yet been processed
    // because `len % 2 != 0`, or B) all elements have been processed except the gap value that was
    // saved at the beginning with `ptr::read(v_base)`. In the case A) the loop will iterate twice,
    // first performing loop_body to take care of the last element that didn't fit into the unroll.
    // After that the behavior is the same as for B) where we use the saved value as `right` to
    // overwrite the duplicate. If this very last call to `is_less` panics the saved value will be
    // copied back including all possible changes via interior mutability. If `is_less` does not
    // panic and the code continues we overwrite the duplicate and do `right = right.add(1)`, this
    // is safe to do with `&mut *gap.value` because `T` is the same as `[T; 1]` and generating a
    // pointer one past the allocation is safe.
    unsafe {
        let mut lt_count = 0;
        let mut right = v_base.add(1);

        let mut gap = GapGuard {
            pos: v_base,
            value: ManuallyDrop::new(ptr::read(v_base)),
        };

        macro_rules! loop_body {
            () => {{
                let right_is_lt = is_less(&*right, pivot);
                let left = v_base.add(lt_count);

                ptr::copy(left, gap.pos, 1);
                ptr::copy_nonoverlapping(right, left, 1);

                gap.pos = right;
                lt_count += right_is_lt as usize;

                right = right.add(1);
                _ = right;
            }};
        }

        let unroll_end = v_base.add(len - (unroll_len - 1));
        while right < unroll_end {
            for _ in 0..unroll_len {
                loop_body!();
            }
        }

        let end = v_base.add(len);
        loop {
            let is_done = right == end;
            right = if is_done { &mut *gap.value } else { right };

            loop_body!();

            if is_done {
                mem::forget(gap);
                break;
            }
        }

        lt_count
    }
}
