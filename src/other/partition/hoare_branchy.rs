use std::ptr;

partition_impl!("hoare_branchy");

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: The unsafety below involves indexing an array. For the first one: We already do
    // the bounds checking here with `l < r`. For the second one: We initially have `l == 0` and
    // `r == v.len()` and we checked that `l < r` at every indexing operation.
    //
    // From here we know that `r` must be at least `r == l` which was shown to be valid from the
    // first one.
    unsafe {
        let arr_ptr = v.as_mut_ptr();

        let mut l = arr_ptr;
        let mut r = arr_ptr.add(v.len());

        loop {
            // Find the first element greater than the pivot.
            while l < r && is_less(&*l, pivot) {
                l = l.add(1);
            }

            // Find the last element equal to the pivot.
            while l < r && !is_less(&*r.sub(1), pivot) {
                r = r.sub(1);
            }
            r = r.sub(1);

            // Are we done?
            if l >= r {
                break;
            }

            // Swap the found pair of out-of-order elements.
            ptr::swap_nonoverlapping(l, r, 1);
            l = l.add(1);
        }

        l.sub_ptr(arr_ptr)
    }
}
