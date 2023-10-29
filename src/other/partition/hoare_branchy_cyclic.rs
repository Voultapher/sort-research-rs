use core::mem::ManuallyDrop;
use core::ptr;

partition_impl!("hoare_branchy_cyclic");

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

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F: FnMut(&T, &T) -> bool>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize {
    let len = v.len();

    if len == 0 {
        return 0;
    }

    // Optimized for large types that are expensive to move. Not optimized for integers. Optimized
    // for small code-gen, assuming that is_less is an expensive operation that generates
    // substantial amounts of code or a call. And that copying elements will likely be a call to
    // memcpy. Using 2 `ptr::copy_nonoverlapping` has the chance to be faster than
    // `ptr::swap_nonoverlapping` because `memcpy` can use wide SIMD based on runtime feature
    // detection. Benchmarks support this analysis.

    let mut gap_opt: Option<GapGuard<T>> = None;

    // SAFETY: The left-to-right scanning loop performs a bounds check, where we know that `left >=
    // v_base && left < right && right <= v_base.add(len)`. The right-to-left scanning loop performs
    // a bounds check ensuring that `right` is in-bounds. We checked that `len` is more than zero,
    // which means that unconditional `right = right.sub(1)` is safe to do. The exit check makes
    // sure that `left` and `right` never alias, making `ptr::copy_nonoverlapping` safe. The
    // drop-guard `gap` ensures that should `is_less` panic we always overwrite the duplicate in the
    // input. `gap.pos` stores the previous value of `right` and starts at `right` and so it too is
    // in-bounds. We never pass the saved `gap.value` to `is_less` while it is inside the `GapGuard`
    // thus any changes via interior mutability will be observed.
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

            // Swap the found pair of out-of-order elements via cyclic permutation.
            let is_first_swap_pair = gap_opt.is_none();

            if is_first_swap_pair {
                gap_opt = Some(GapGuard {
                    pos: right,
                    value: ManuallyDrop::new(ptr::read(left)),
                });
            }

            let gap = gap_opt.as_mut().unwrap_unchecked();

            // Single place where we instantiate ptr::copy_nonoverlapping in the partition.
            if !is_first_swap_pair {
                ptr::copy_nonoverlapping(left, gap.pos, 1);
            }
            gap.pos = right;
            ptr::copy_nonoverlapping(right, left, 1);

            left = left.add(1);
        }

        left.sub_ptr(v_base)

        // `gap_opt` goes out of scope and overwrites the last wrong-side element on the right side
        // with the first wrong-side element of the left side that was initially overwritten by the
        // first wrong-side element on the right side.
    }
}
