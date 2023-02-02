// use std::ptr;

use crate::unstable::rust_ipn::branchless_swap;

partition_impl!("simple_scan_branchless");

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    let arr_ptr = v.as_mut_ptr();

    unsafe {
        let mut l_ptr = arr_ptr;
        let mut r_ptr = arr_ptr.add(len - 1);

        while l_ptr < r_ptr {
            let elem_is_less = is_less(&*l_ptr, pivot);
            branchless_swap(l_ptr, r_ptr, !elem_is_less);

            l_ptr = l_ptr.add(elem_is_less as usize);
            r_ptr = r_ptr.offset(-(!elem_is_less as isize));
        }

        // Some final cleanup is missing here.

        l_ptr.offset_from(arr_ptr) as usize
    }
}
