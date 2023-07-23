use core::mem::ManuallyDrop;
use core::ptr;

partition_impl!("hoare_branchy_cyclic");

struct GapGuardNonoverlapping<T> {
    pos: *mut T,
    value: ManuallyDrop<T>,
}

impl<T> Drop for GapGuardNonoverlapping<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::write(self.pos, ManuallyDrop::take(&mut self.value));
        }
    }
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // Optimized for large types that are expensive to move. Not optimized for integers. Optimized
    // for small code-gen, assuming that is_less is an expensive operation that generates
    // substantial amounts of code or a call. And that copying elements will likely be a call to
    // memcpy. Using 2 `ptr::copy_nonoverlapping` has the chance to be faster than
    // `ptr::swap_nonoverlapping` because `memcpy` can use wide SIMD based on runtime feature
    // detection. Benchmarks support this analysis.

    let mut gap_guard_opt: Option<GapGuardNonoverlapping<T>> = None;

    // SAFETY: The unsafety below involves indexing an array. For the first one: We already do
    // the bounds checking here with `l < r`. For the second one: We initially have `l == 0` and
    // `r == v.len()` and we checked that `l < r` at every indexing operation.
    //
    // From here we know that `r` must be at least `r == l` which was shown to be valid from the
    // first one.
    unsafe {
        let arr_ptr = v.as_mut_ptr();

        let mut l_ptr = arr_ptr;
        let mut r_ptr = arr_ptr.add(v.len());

        loop {
            // Find the first element greater than the pivot.
            while l_ptr < r_ptr && is_less(&*l_ptr, pivot) {
                l_ptr = l_ptr.add(1);
            }

            // Find the last element equal to the pivot.
            while l_ptr < r_ptr && !is_less(&*r_ptr.sub(1), pivot) {
                r_ptr = r_ptr.sub(1);
            }
            r_ptr = r_ptr.sub(1);

            // Are we done?
            if l_ptr >= r_ptr {
                assert!(l_ptr != r_ptr);
                break;
            }

            // Swap the found pair of out-of-order elements via cyclic permutation.
            let is_first_swap_pair = gap_guard_opt.is_none();

            if is_first_swap_pair {
                gap_guard_opt = Some(GapGuardNonoverlapping {
                    pos: r_ptr,
                    value: ManuallyDrop::new(ptr::read(l_ptr)),
                });
            }

            let gap_guard = gap_guard_opt.as_mut().unwrap_unchecked();

            // Single place where we instantiate ptr::copy_nonoverlapping in the partition.
            if !is_first_swap_pair {
                ptr::copy_nonoverlapping(l_ptr, gap_guard.pos, 1);
            }
            gap_guard.pos = r_ptr;
            ptr::copy_nonoverlapping(r_ptr, l_ptr, 1);

            l_ptr = l_ptr.add(1);
        }

        l_ptr.sub_ptr(arr_ptr)

        // `gap_guard_opt` goes out of scope and overwrites the last right wrong-side element with
        // the first left wrong-side element that was initially overwritten by the first right
        // wrong-side element.
    }
}
