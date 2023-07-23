use core::ptr;

partition_impl!("lomuto_branchless");

fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    let arr_ptr = v.as_mut_ptr();

    // SAFETY: TODO
    unsafe {
        let mut left_ptr = arr_ptr;

        for i in 0..len {
            let right_ptr = arr_ptr.add(i);

            if is_less(&*right_ptr, pivot) {
                ptr::swap(left_ptr, right_ptr);
                left_ptr = left_ptr.add(1);
            }
        }

        left_ptr.sub_ptr(arr_ptr)
    }
}
