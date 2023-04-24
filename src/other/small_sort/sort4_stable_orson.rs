use core::cmp::Ordering;
use core::mem::MaybeUninit;
use core::ptr;

sort_impl!("sort4_stable_orson");

/// SAFETY: The caller MUST guarantee that `arr_ptr` is valid for 4 reads and `dest_ptr` is valid
/// for 4 writes.
#[inline(never)]
pub unsafe fn sort4_stable<T, F>(arr_ptr: *const T, dest_ptr: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // By limiting select to picking pointers, we are guaranteed good cmov code-gen regardless of
    // type T layout. Further this only does 5 instead of 6 comparisons compared to a stable
    // transposition 4 element sorting-network. Also by only operating on pointers, we get optimal
    // element copy usage. Doing exactly 1 copy per element.

    // let arr_ptr = v.as_ptr();

    unsafe {
        // Stably create two pairs a <= b and c <= d.
        let c1 = is_less(&*arr_ptr.add(1), &*arr_ptr) as usize;
        let c2 = is_less(&*arr_ptr.add(3), &*arr_ptr.add(2)) as usize;
        let a = arr_ptr.add(c1);
        let b = arr_ptr.add(c1 ^ 1);
        let c = arr_ptr.add(2 + c2);
        let d = arr_ptr.add(2 + (c2 ^ 1));

        // Compare (a, c) and (b, d) to identify max/min. We're left with two
        // unknown elements, but because we are a stable sort we must know which
        // one is leftmost and which one is rightmost.
        // c3, c4 | min max unknown_left unknown_right
        //  0,  0 |  a   d    b         c
        //  0,  1 |  a   b    c         d
        //  1,  0 |  c   d    a         b
        //  1,  1 |  c   b    a         d
        let c3 = is_less(&*c, &*a);
        let c4 = is_less(&*d, &*b);
        let min = select(c3, c, a);
        let max = select(c4, b, d);
        let unknown_left = select(c3, a, select(c4, c, b));
        let unknown_right = select(c4, d, select(c3, b, c));

        // Sort the last two unknown elements.
        let c5 = is_less(&*unknown_right, &*unknown_left);
        let lo = select(c5, unknown_right, unknown_left);
        let hi = select(c5, unknown_left, unknown_right);

        ptr::copy_nonoverlapping(min, dest_ptr, 1);
        ptr::copy_nonoverlapping(lo, dest_ptr.add(1), 1);
        ptr::copy_nonoverlapping(hi, dest_ptr.add(2), 1);
        ptr::copy_nonoverlapping(max, dest_ptr.add(3), 1);
    }

    #[inline(always)]
    pub fn select<T>(cond: bool, if_true: *const T, if_false: *const T) -> *const T {
        if cond {
            if_true
        } else {
            if_false
        }
    }
}

fn sort_impl<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let mut scratch = MaybeUninit::<[T; 4]>::uninit();
    let scratch_ptr = scratch.as_mut_ptr() as *mut T;

    unsafe {
        sort4_stable(v.as_ptr(), scratch_ptr, is_less);
        ptr::copy_nonoverlapping(scratch_ptr, v.as_mut_ptr(), 4);
    }
}

fn sort<T: Ord>(v: &mut [T]) {
    sort_impl(v, &mut |a, b| a.lt(b));
}

fn sort_by<T, F>(v: &mut [T], mut compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    sort_impl(v, &mut |a, b| compare(a, b) == Ordering::Less);
}
