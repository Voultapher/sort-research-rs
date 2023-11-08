use core::ptr;

partition_impl!("lomuto_branchy");

fn partition<T, F: FnMut(&T, &T) -> bool>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize {
    let len = v.len();
    let v_base = v.as_mut_ptr();

    // SAFETY: The bounded loop ensures that `right` is always in-bounds. `v` and `pivot` can't
    // alias because of type system rules. `left` is guaranteed somewhere between `v_base` and
    // `right` making it also in-bounds and the call to `sub_ptr` at the end safe.
    unsafe {
        let mut left = v_base;

        for i in 0..len {
            let right = v_base.add(i);
            let right_is_lt = is_less(&*right, pivot);

            if right_is_lt {
                ptr::swap(left, right);
                left = left.add(1);
            }
        }

        left.sub_ptr(v_base)
    }
}
