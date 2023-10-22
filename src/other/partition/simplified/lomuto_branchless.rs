use core::mem::ManuallyDrop;
use core::ptr;

/// Swap two values pointed to by `x` and `y` if `should_swap` is true.
#[inline(always)]
pub unsafe fn branchless_swap<T>(x: *mut T, y: *mut T, should_swap: bool) {
    // This is a branchless version of swap if.
    // The equivalent code with a branch would be:
    //
    // if should_swap {
    //     ptr::swap(x, y);
    // }

    // SAFETY: the caller must guarantee that `x` and `y` are valid for writes
    // and properly aligned.
    unsafe {
        // The goal is to generate cmov instructions here.
        let x_swap = if should_swap { y } else { x };
        let y_swap = if should_swap { x } else { y };

        let y_swap_copy = ManuallyDrop::new(ptr::read(y_swap));

        ptr::copy(x_swap, x, 1);
        ptr::copy_nonoverlapping(&*y_swap_copy, y, 1);
    }
}

fn partition<T, F: FnMut(&T, &T) -> bool>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize {
    let len = v.len();
    let v_base = v.as_mut_ptr();

    // SAFETY: The bounded loop ensures that `right` is always in-bounds. `v` and `pivot` can't
    // alias because of type system rules. The left side element `left` can only be incremented once
    // per iteration, so it is <= `right` which makes it in-bounds as a transitive property. From
    // this also follows that the call to `sub_ptr` at the end is safe.
    unsafe {
        let mut left = v_base;

        for i in 0..len {
            let right = v_base.add(i);
            let right_is_lt = is_less(&*right, pivot);
            branchless_swap(left, right, right_is_lt);
            left = left.add(right_is_lt as usize);
        }

        left.sub_ptr(v_base)
    }
}
