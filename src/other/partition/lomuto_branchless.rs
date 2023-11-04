use core::mem::{self, ManuallyDrop};
use core::ptr;

partition_impl!("lomuto_branchless");

/// Swap two values in array pointed to by a and b if b is less than a.
#[inline(always)]
pub unsafe fn branchless_swap<T>(x: *mut T, y: *mut T, should_swap: bool) {
    // SAFETY: the caller must guarantee that `x` and `y` are valid for writes and properly aligned,
    // and part of the same allocation.

    // This is a branchless version of swap if.
    // The equivalent code with a branch would be:
    //
    // if should_swap {
    //     ptr::swap(x, y);
    // }

    // The goal is to generate cmov instructions here.
    let x_swap = if should_swap { y } else { x };
    let y_swap = if should_swap { x } else { y };

    let y_swap_copy = ManuallyDrop::new(ptr::read(y_swap));

    ptr::copy(x_swap, x, 1);
    ptr::copy_nonoverlapping(&*y_swap_copy, y, 1);
}

fn partition<T, F: FnMut(&T, &T) -> bool>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize {
    let len = v.len();
    let v_base = v.as_mut_ptr();

    // Manually unrolled as micro-optimization as only x86 gets auto-unrolling but not Arm.
    let unroll_len = if const { mem::size_of::<T>() <= 16 } {
        2
    } else {
        1
    };

    // SAFETY: The bounded loop ensures that `right` is always in-bounds. `v` and `pivot` can't
    // alias because of type system rules. `left` is guaranteed somewhere between `v_base` and
    // `right` making it also in-bounds and the call to `sub_ptr` at the end safe.
    unsafe {
        let mut lt_count = 0;
        let mut right = v_base;

        macro_rules! loop_body {
            () => {{
                let right_is_lt = is_less(&*right, pivot);

                let left = v_base.add(lt_count);
                branchless_swap(left, right, right_is_lt);

                lt_count += right_is_lt as usize;

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

        lt_count
    }
}
