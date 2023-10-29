use std::ptr;

partition_impl!("hoare_branchy");

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F: FnMut(&T, &T) -> bool>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize {
    let len = v.len();

    if len == 0 {
        return 0;
    }

    // SAFETY: The left-to-right scanning loop performs a bounds check, where we know that `left >=
    // v_base && left < right && right <= v_base.add(len)`. The right-to-left scanning loop performs
    // a bounds check ensuring that `right` is in-bounds. We checked that `len` is more than zero,
    // which means that unconditional `right = right.sub(1)` is safe to do. The exit check makes
    // sure that `left` and `right` never alias, making `ptr::swap_nonoverlapping` safe.
    unsafe {
        let v_base = v.as_mut_ptr();

        let mut left = v_base;
        let mut right = v_base.add(len);

        loop {
            // Find the first element greater than the pivot.
            while left < right && is_less(&*left, pivot) {
                left = left.add(1);
            }

            // Find the last element equal to the pivot.
            loop {
                right = right.sub(1);
                if left >= right || is_less(&*right, pivot) {
                    break;
                }
            }

            if left >= right {
                break;
            }

            ptr::swap_nonoverlapping(left, right, 1);
            left = left.add(1);
        }

        left.sub_ptr(v_base)
    }
}
