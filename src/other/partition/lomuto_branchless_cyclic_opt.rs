//! Same idea as lomuto_branchless_cyclic but refined by Orson Peters to avoid the cmov.

use core::mem::ManuallyDrop;
use core::ptr;

partition_impl!("lomuto_branchless_cyclic_opt");

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

fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // Manually unrolled to ensure consistent performance across various targets.
    const UNROLL_LEN: usize = 2;

    let len = v.len();
    if len == 0 {
        return 0;
    }

    unsafe {
        let arr_ptr = v.as_mut_ptr();

        let mut gap = GapGuard {
            pos: arr_ptr,
            value: ManuallyDrop::new(ptr::read(arr_ptr)),
        };

        let end = arr_ptr.add(len);
        let mut lt_count = 0;
        while gap.pos.wrapping_add(UNROLL_LEN) < end {
            for _ in 0..UNROLL_LEN {
                let is_next_lt = is_less(&*gap.pos.add(1), pivot);
                ptr::copy(arr_ptr.add(lt_count), gap.pos, 1);
                ptr::copy_nonoverlapping(gap.pos.add(1), arr_ptr.add(lt_count), 1);
                gap.pos = gap.pos.add(1);
                lt_count += is_next_lt as usize;
            }
        }

        let mut scan = gap.pos;
        drop(gap);

        while scan < end {
            let is_lomuto_less = is_less(&*scan, pivot);
            ptr::swap(arr_ptr.add(lt_count), scan);
            scan = scan.add(1);
            lt_count += is_lomuto_less as usize;
        }

        lt_count
    }
}
