//! Inspired by https://github.com/zeux/nanosort

use core::mem;
use core::ptr;

partition_impl!("lomuto_nanosort");

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
    // `right`.
    unsafe {
        let mut right = v_base;
        let mut lt_count = 0;

        macro_rules! loop_body {
            () => {{
                let right_is_lt = is_less(&*right, pivot);
                ptr::swap(v_base.add(lt_count), right);
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
