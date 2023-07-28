use core::intrinsics;
use core::mem::{self, MaybeUninit};
use core::ptr;

use crate::{has_efficient_in_place_swap, Freeze, GapGuardNonoverlapping, IsTrue};

// Use a trait to focus code-gen on only the parts actually relevant for the type. Avoid generating
// LLVM-IR for the sorting-network and median-networks for types that don't qualify.
pub(crate) trait SmallSortImpl: Sized {
    const MAX_SMALL_SORT_LEN: usize;

    /// Sorts `v` using strategies optimized for small sizes.
    fn small_sort<F>(v: &mut [Self], is_less: &mut F)
    where
        F: FnMut(&Self, &Self) -> bool;
}

impl<T> SmallSortImpl for T {
    default const MAX_SMALL_SORT_LEN: usize = 20;

    default fn small_sort<F>(v: &mut [Self], is_less: &mut F)
    where
        F: FnMut(&Self, &Self) -> bool,
    {
        if v.len() >= 2 {
            insertion_sort_shift_left(v, 1, is_less);
        }
    }
}

impl<T: Freeze> SmallSortImpl for T {
    default const MAX_SMALL_SORT_LEN: usize = 20;

    default fn small_sort<F>(v: &mut [Self], is_less: &mut F)
    where
        F: FnMut(&Self, &Self) -> bool,
    {
        small_sort_general(v, is_less);
    }
}

impl<T> SmallSortImpl for T
where
    T: Freeze + Copy,
    (): IsTrue<{ has_efficient_in_place_swap::<T>() }>,
{
    const MAX_SMALL_SORT_LEN: usize = 32;

    fn small_sort<F>(v: &mut [Self], is_less: &mut F)
    where
        F: FnMut(&Self, &Self) -> bool,
    {
        // I suspect that generalized efficient indirect branchless sorting constructs like
        // sort4_indirect for larger sizes exist. But finding them is an open research problem.
        // And even then it's not clear that they would be better than in-place sorting-networks
        // as used in small_sort_network.
        small_sort_network(v, is_less);
    }
}

#[inline(always)]
unsafe fn merge_up<T, F>(
    mut src_left: *const T,
    mut src_right: *const T,
    mut dest_ptr: *mut T,
    is_less: &mut F,
) -> (*const T, *const T, *mut T)
where
    F: FnMut(&T, &T) -> bool,
{
    // This is a branchless merge utility function.
    // The equivalent code with a branch would be:
    //
    // if !is_less(&*src_right, &*src_left) {
    //     ptr::copy_nonoverlapping(src_left, dest_ptr, 1);
    //     src_left = src_left.wrapping_add(1);
    // } else {
    //     ptr::copy_nonoverlapping(src_right, dest_ptr, 1);
    //     src_right = src_right.wrapping_add(1);
    // }
    // dest_ptr = dest_ptr.add(1);

    // SAFETY: The caller must guarantee that `src_left`, `src_right` are valid to read and
    // `dest_ptr` is valid to write, while not aliasing.
    unsafe {
        let is_l = !is_less(&*src_right, &*src_left);
        let copy_ptr = if is_l { src_left } else { src_right };
        ptr::copy_nonoverlapping(copy_ptr, dest_ptr, 1);
        src_right = src_right.wrapping_add(!is_l as usize);
        src_left = src_left.wrapping_add(is_l as usize);
        dest_ptr = dest_ptr.add(1);
    }

    (src_left, src_right, dest_ptr)
}

#[inline(always)]
unsafe fn merge_down<T, F>(
    mut src_left: *const T,
    mut src_right: *const T,
    mut dest_ptr: *mut T,
    is_less: &mut F,
) -> (*const T, *const T, *mut T)
where
    F: FnMut(&T, &T) -> bool,
{
    // This is a branchless merge utility function.
    // The equivalent code with a branch would be:
    //
    // if !is_less(&*src_right, &*src_left) {
    //     ptr::copy_nonoverlapping(src_right, dest_ptr, 1);
    //     src_right = src_right.wrapping_sub(1);
    // } else {
    //     ptr::copy_nonoverlapping(src_left, dest_ptr, 1);
    //     src_left = src_left.wrapping_sub(1);
    // }
    // dest_ptr = dest_ptr.sub(1);

    // SAFETY: The caller must guarantee that `src_left`, `src_right` are valid to read and
    // `dest_ptr` is valid to write, while not aliasing.
    unsafe {
        let is_l = !is_less(&*src_right, &*src_left);
        let copy_ptr = if is_l { src_right } else { src_left };
        ptr::copy_nonoverlapping(copy_ptr, dest_ptr, 1);
        src_right = src_right.wrapping_sub(is_l as usize);
        src_left = src_left.wrapping_sub(!is_l as usize);
        dest_ptr = dest_ptr.sub(1);
    }

    (src_left, src_right, dest_ptr)
}

/// Merge v assuming the len is even and v[..len / 2] and v[len / 2..] are sorted.
///
/// Original idea for bi-directional merging by Igor van den Hoven (quadsort), adapted to only use
/// merge up and down. In contrast to the original parity_merge function, it performs 2 writes
/// instead of 4 per iteration. Ord violation detection was added.
unsafe fn bi_directional_merge_even<T, F>(v: &[T], dest_ptr: *mut T, is_less: &mut F)
where
    T: Freeze,
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: the caller must guarantee that `dest_ptr` is valid for v.len() writes.
    // Also `v.as_ptr` and `dest_ptr` must not alias.
    //
    // The caller must guarantee that T cannot modify itself inside is_less.
    // merge_up and merge_down read left and right pointers and potentially modify the stack value
    // they point to, if T has interior mutability. This may leave one or two potential writes to
    // the stack value un-observed when dest is copied onto of src.

    // It helps to visualize the merge:
    //
    // Initial:
    //
    //  |ptr_data (in dest)
    //  |ptr_left           |ptr_right
    //  v                   v
    // [xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx]
    //                     ^                   ^
    //                     |t_ptr_left         |t_ptr_right
    //                                         |t_ptr_data (in dest)
    //
    // After:
    //
    //                      |ptr_data (in dest)
    //        |ptr_left     |           |ptr_right
    //        v             v           v
    // [xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx]
    //       ^             ^           ^
    //       |t_ptr_left   |           |t_ptr_right
    //                     |t_ptr_data (in dest)
    //
    //
    // Note, the pointers that have been written, are now one past where they were read and
    // copied. written == incremented or decremented + copy to dest.

    let len = v.len();
    let src_ptr = v.as_ptr();

    let len_div_2 = len / 2;

    // SAFETY: No matter what the result of the user-provided comparison function is, all 4 read
    // pointers will always be in-bounds. Writing `ptr_data` and `t_ptr_data` will always be in
    // bounds if the caller guarantees that `dest_ptr` is valid for `v.len()` writes.
    unsafe {
        let mut ptr_left = src_ptr;
        let mut ptr_right = src_ptr.wrapping_add(len_div_2);
        let mut ptr_data = dest_ptr;

        let mut t_ptr_left = src_ptr.wrapping_add(len_div_2 - 1);
        let mut t_ptr_right = src_ptr.wrapping_add(len - 1);
        let mut t_ptr_data = dest_ptr.wrapping_add(len - 1);

        for _ in 0..len_div_2 {
            (ptr_left, ptr_right, ptr_data) = merge_up(ptr_left, ptr_right, ptr_data, is_less);
            (t_ptr_left, t_ptr_right, t_ptr_data) =
                merge_down(t_ptr_left, t_ptr_right, t_ptr_data, is_less);
        }

        let left_diff = (ptr_left as usize).wrapping_sub(t_ptr_left as usize);
        let right_diff = (ptr_right as usize).wrapping_sub(t_ptr_right as usize);

        if !(left_diff == mem::size_of::<T>() && right_diff == mem::size_of::<T>()) {
            panic_on_ord_violation();
        }
    }
}

/// Swap two values in array pointed to by a_ptr and b_ptr if b is less than a.
#[inline(always)]
pub unsafe fn branchless_swap<T>(a_ptr: *mut T, b_ptr: *mut T, should_swap: bool) {
    // SAFETY: the caller must guarantee that `a_ptr` and `b_ptr` are valid for writes
    // and properly aligned, and part of the same allocation.

    // This is a branchless version of swap if.
    // The equivalent code with a branch would be:
    //
    // if should_swap {
    //     ptr::swap(a_ptr, b_ptr, 1);
    // }

    // Give ourselves some scratch space to work with.
    // We do not have to worry about drops: `MaybeUninit` does nothing when dropped.
    let mut tmp = MaybeUninit::<T>::uninit();

    // The goal is to generate cmov instructions here.
    let a_swap_ptr = if should_swap { b_ptr } else { a_ptr };
    let b_swap_ptr = if should_swap { a_ptr } else { b_ptr };

    ptr::copy_nonoverlapping(b_swap_ptr, tmp.as_mut_ptr(), 1);
    ptr::copy(a_swap_ptr, a_ptr, 1);
    ptr::copy_nonoverlapping(tmp.as_ptr(), b_ptr, 1);
}

/// Swap two values in array pointed to by a_ptr and b_ptr if b is less than a.
pub unsafe fn swap_if_less<T, F>(arr_ptr: *mut T, a: usize, b: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: the caller must guarantee that `a` and `b` each added to `arr_ptr` yield valid
    // pointers into `arr_ptr`, and are properly aligned, and part of the same allocation.

    let a_ptr = arr_ptr.add(a);
    let b_ptr = arr_ptr.add(b);

    // PANIC SAFETY: if is_less panics, no scratch memory was created and the slice should still be
    // in a well defined state, without duplicates.

    // Important to only swap if it is more and not if it is equal. is_less should return false for
    // equal, so we don't swap.
    let should_swap = is_less(&*b_ptr, &*a_ptr);
    branchless_swap(a_ptr, b_ptr, should_swap);
}

// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
// performance impact.
#[inline(never)]
fn sort9_optimal<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >=9.
    if v.len() != 9 {
        intrinsics::abort();
    }

    let arr_ptr = v.as_mut_ptr();

    // Optimal sorting network see:
    // https://bertdobbelaere.github.io/sorting_networks.html.

    // We checked the len.
    unsafe {
        swap_if_less(arr_ptr, 0, 3, is_less);
        swap_if_less(arr_ptr, 1, 7, is_less);
        swap_if_less(arr_ptr, 2, 5, is_less);
        swap_if_less(arr_ptr, 4, 8, is_less);
        swap_if_less(arr_ptr, 0, 7, is_less);
        swap_if_less(arr_ptr, 2, 4, is_less);
        swap_if_less(arr_ptr, 3, 8, is_less);
        swap_if_less(arr_ptr, 5, 6, is_less);
        swap_if_less(arr_ptr, 0, 2, is_less);
        swap_if_less(arr_ptr, 1, 3, is_less);
        swap_if_less(arr_ptr, 4, 5, is_less);
        swap_if_less(arr_ptr, 7, 8, is_less);
        swap_if_less(arr_ptr, 1, 4, is_less);
        swap_if_less(arr_ptr, 3, 6, is_less);
        swap_if_less(arr_ptr, 5, 7, is_less);
        swap_if_less(arr_ptr, 0, 1, is_less);
        swap_if_less(arr_ptr, 2, 4, is_less);
        swap_if_less(arr_ptr, 3, 5, is_less);
        swap_if_less(arr_ptr, 6, 8, is_less);
        swap_if_less(arr_ptr, 2, 3, is_less);
        swap_if_less(arr_ptr, 4, 5, is_less);
        swap_if_less(arr_ptr, 6, 7, is_less);
        swap_if_less(arr_ptr, 1, 2, is_less);
        swap_if_less(arr_ptr, 3, 4, is_less);
        swap_if_less(arr_ptr, 5, 6, is_less);
    }
}

// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
// performance impact.
#[inline(never)]
fn sort13_optimal<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 13.
    if v.len() != 13 {
        intrinsics::abort();
    }

    let arr_ptr = v.as_mut_ptr();

    // Optimal sorting network see:
    // https://bertdobbelaere.github.io/sorting_networks.html.

    // We checked the len.
    unsafe {
        swap_if_less(arr_ptr, 0, 12, is_less);
        swap_if_less(arr_ptr, 1, 10, is_less);
        swap_if_less(arr_ptr, 2, 9, is_less);
        swap_if_less(arr_ptr, 3, 7, is_less);
        swap_if_less(arr_ptr, 5, 11, is_less);
        swap_if_less(arr_ptr, 6, 8, is_less);
        swap_if_less(arr_ptr, 1, 6, is_less);
        swap_if_less(arr_ptr, 2, 3, is_less);
        swap_if_less(arr_ptr, 4, 11, is_less);
        swap_if_less(arr_ptr, 7, 9, is_less);
        swap_if_less(arr_ptr, 8, 10, is_less);
        swap_if_less(arr_ptr, 0, 4, is_less);
        swap_if_less(arr_ptr, 1, 2, is_less);
        swap_if_less(arr_ptr, 3, 6, is_less);
        swap_if_less(arr_ptr, 7, 8, is_less);
        swap_if_less(arr_ptr, 9, 10, is_less);
        swap_if_less(arr_ptr, 11, 12, is_less);
        swap_if_less(arr_ptr, 4, 6, is_less);
        swap_if_less(arr_ptr, 5, 9, is_less);
        swap_if_less(arr_ptr, 8, 11, is_less);
        swap_if_less(arr_ptr, 10, 12, is_less);
        swap_if_less(arr_ptr, 0, 5, is_less);
        swap_if_less(arr_ptr, 3, 8, is_less);
        swap_if_less(arr_ptr, 4, 7, is_less);
        swap_if_less(arr_ptr, 6, 11, is_less);
        swap_if_less(arr_ptr, 9, 10, is_less);
        swap_if_less(arr_ptr, 0, 1, is_less);
        swap_if_less(arr_ptr, 2, 5, is_less);
        swap_if_less(arr_ptr, 6, 9, is_less);
        swap_if_less(arr_ptr, 7, 8, is_less);
        swap_if_less(arr_ptr, 10, 11, is_less);
        swap_if_less(arr_ptr, 1, 3, is_less);
        swap_if_less(arr_ptr, 2, 4, is_less);
        swap_if_less(arr_ptr, 5, 6, is_less);
        swap_if_less(arr_ptr, 9, 10, is_less);
        swap_if_less(arr_ptr, 1, 2, is_less);
        swap_if_less(arr_ptr, 3, 4, is_less);
        swap_if_less(arr_ptr, 5, 7, is_less);
        swap_if_less(arr_ptr, 6, 8, is_less);
        swap_if_less(arr_ptr, 2, 3, is_less);
        swap_if_less(arr_ptr, 4, 5, is_less);
        swap_if_less(arr_ptr, 6, 7, is_less);
        swap_if_less(arr_ptr, 8, 9, is_less);
        swap_if_less(arr_ptr, 3, 4, is_less);
        swap_if_less(arr_ptr, 5, 6, is_less);
    }
}

fn sort13_plus<T, F>(v: &mut [T], is_less: &mut F)
where
    T: Freeze,
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    const MAX_BRANCHLESS_SMALL_SORT: usize = i32::MAX_SMALL_SORT_LEN;

    if len < 13 || len > MAX_BRANCHLESS_SMALL_SORT {
        intrinsics::abort();
    }

    if len < 18 {
        sort13_optimal(&mut v[0..13], is_less);
        insertion_sort_shift_left(v, 13, is_less);
        return;
    }

    // This should optimize to a shift right https://godbolt.org/z/vYGsznPPW.
    let even_len = len - (len % 2 != 0) as usize;
    let len_div_2 = even_len / 2;

    let mid = if len < 26 {
        sort9_optimal(&mut v[0..9], is_less);
        sort9_optimal(&mut v[len_div_2..(len_div_2 + 9)], is_less);

        9
    } else {
        sort13_optimal(&mut v[0..13], is_less);
        sort13_optimal(&mut v[len_div_2..(len_div_2 + 13)], is_less);

        13
    };

    insertion_sort_shift_left(&mut v[0..len_div_2], mid, is_less);
    insertion_sort_shift_left(&mut v[len_div_2..], mid, is_less);

    let mut swap = MaybeUninit::<[T; MAX_BRANCHLESS_SMALL_SORT]>::uninit();
    let swap_ptr = swap.as_mut_ptr() as *mut T;

    // SAFETY: We checked that T is Freeze and thus observation safe.
    // Should is_less panic v was not modified in parity_merge and retains it's original input.
    // swap and v must not alias and swap has v.len() space.
    unsafe {
        bi_directional_merge_even(&mut v[..even_len], swap_ptr, is_less);
        ptr::copy_nonoverlapping(swap_ptr, v.as_mut_ptr(), even_len);
    }

    if len != even_len {
        // SAFETY: We know len >= 2.
        unsafe {
            insert_tail(v, is_less);
        }
    }
}

fn small_sort_network<T, F>(v: &mut [T], is_less: &mut F)
where
    T: Freeze,
    F: FnMut(&T, &T) -> bool,
{
    // This implementation is tuned to be efficient for integer types.

    let len = v.len();

    // Always sort assuming somewhat random distribution.
    // Patterns should have already been found by the other analysis steps.
    //
    // Small total slices are handled separately, see function quicksort.
    if len >= 13 {
        sort13_plus(v, is_less);
    } else if len >= 2 {
        let end = if len >= 9 {
            sort9_optimal(&mut v[0..9], is_less);
            9
        } else {
            1
        };

        insertion_sort_shift_left(v, end, is_less);
    }
}

fn small_sort_general<T, F>(v: &mut [T], is_less: &mut F)
where
    T: Freeze,
    F: FnMut(&T, &T) -> bool,
{
    // This implementation is tuned to be efficient for various types that are larger than u64.

    const MAX_SIZE: usize = String::MAX_SMALL_SORT_LEN;

    let len = v.len();

    let mut scratch = MaybeUninit::<[T; MAX_SIZE]>::uninit();
    let scratch_ptr = scratch.as_mut_ptr() as *mut T;

    if len >= 16 && len <= MAX_SIZE {
        let even_len = len - (len % 2);
        let len_div_2 = even_len / 2;

        // SAFETY: scratch_ptr is valid and has enough space. And we checked that both
        // v[..len_div_2] and v[len_div_2..] are at least 8 large.
        unsafe {
            let arr_ptr = v.as_mut_ptr();
            sort8_indirect(arr_ptr, scratch_ptr, is_less);
            sort8_indirect(arr_ptr.add(len_div_2), scratch_ptr, is_less);
        }

        insertion_sort_shift_left(&mut v[0..len_div_2], 8, is_less);
        insertion_sort_shift_left(&mut v[len_div_2..], 8, is_less);

        // SAFETY: We checked that T is Freeze and thus observation safe. Should is_less panic v
        // was not modified in parity_merge and retains it's original input. swap and v must not
        // alias and swap has v.len() space.
        unsafe {
            bi_directional_merge_even(&mut v[..even_len], scratch_ptr, is_less);
            ptr::copy_nonoverlapping(scratch_ptr, v.as_mut_ptr(), even_len);
        }

        if len != even_len {
            // SAFETY: We know len >= 2.
            unsafe {
                insert_tail(v, is_less);
            }
        }
    } else if len >= 2 {
        let offset = if len >= 8 {
            // SAFETY: scratch_ptr is valid and has enough space.
            unsafe {
                sort8_indirect(v.as_mut_ptr(), scratch_ptr, is_less);
            }

            8
        } else {
            1
        };

        insertion_sort_shift_left(v, offset, is_less);
    }
}

/// SAFETY: The caller MUST guarantee that `arr_ptr` is valid for 4 reads and `dest_ptr` is valid
/// for 4 writes.
pub unsafe fn sort4_indirect<T, F>(arr_ptr: *const T, dest_ptr: *mut T, is_less: &mut F)
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

/// SAFETY: The caller MUST guarantee that `arr_ptr` is valid for 8 reads and writes, and
/// `scratch_ptr` is valid for 8 writes.
#[inline(never)]
unsafe fn sort8_indirect<T, F>(arr_ptr: *mut T, scratch_ptr: *mut T, is_less: &mut F)
where
    T: Freeze,
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: The caller must guarantee that scratch_ptr is valid for 8 writes, and that arr_ptr is
    // valid for 8 reads.
    unsafe {
        sort4_indirect(arr_ptr, scratch_ptr, is_less);
        sort4_indirect(arr_ptr.add(4), scratch_ptr.add(4), is_less);
    }

    // SAFETY: We checked that T is Freeze and thus observation safe.
    // Should is_less panic v was not modified in parity_merge and retains its original input.
    // swap and v must not alias and swap has v.len() space.
    unsafe {
        // It's slightly faster to merge directly into v and copy over the 'safe' elements of swap
        // into v only if there was a panic. This technique is also known as ping-pong merge.
        let drop_guard = DropGuard {
            src: scratch_ptr,
            dest: arr_ptr,
        };
        bi_directional_merge_even(
            &*ptr::slice_from_raw_parts(scratch_ptr, 8),
            arr_ptr,
            is_less,
        );
        mem::forget(drop_guard);
    }

    struct DropGuard<T> {
        src: *const T,
        dest: *mut T,
    }

    impl<T> Drop for DropGuard<T> {
        fn drop(&mut self) {
            // SAFETY: `T` is not a zero-sized type, src must hold the original 8 elements of v in
            // any order. And dest must be valid to write 8 elements.
            //
            // Use black_box to emit memcpy instead of efficient direct copying. This reduces the
            // binary size, and this path will only be used if the comparison function panics.
            unsafe {
                ptr::copy_nonoverlapping(self.src, self.dest, 8);
            }
        }
    }
}

// --- Insertion sort ---

/// Inserts `v[v.len() - 1]` into pre-sorted sequence `v[..v.len() - 1]` so that whole `v[..]`
/// becomes sorted.
unsafe fn insert_tail<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    if v.len() < 2 {
        intrinsics::abort();
    }

    let arr_ptr = v.as_mut_ptr();
    let i = v.len() - 1;

    // SAFETY: caller must ensure v is at least len 2.
    unsafe {
        // See insert_head which talks about why this approach is beneficial.
        let i_ptr = arr_ptr.add(i);

        // It's important that we use i_ptr here. If this check is positive and we continue,
        // We want to make sure that no other copy of the value was seen by is_less.
        // Otherwise we would have to copy it back.
        if is_less(&*i_ptr, &*i_ptr.sub(1)) {
            // It's important, that we use tmp for comparison from now on. As it is the value that
            // will be copied back. And notionally we could have created a divergence if we copy
            // back the wrong value.
            // Intermediate state of the insertion process is always tracked by `gap`, which
            // serves two purposes:
            // 1. Protects integrity of `v` from panics in `is_less`.
            // 2. Fills the remaining gap in `v` in the end.
            //
            // Panic safety:
            //
            // If `is_less` panics at any point during the process, `gap` will get dropped and
            // fill the gap in `v` with `tmp`, thus ensuring that `v` still holds every object it
            // initially held exactly once.
            let mut gap = GapGuardNonoverlapping {
                pos: i_ptr.sub(1),
                value: mem::ManuallyDrop::new(ptr::read(i_ptr)),
            };
            ptr::copy_nonoverlapping(gap.pos, i_ptr, 1);

            // SAFETY: We know i is at least 1.
            for j in (0..(i - 1)).rev() {
                let j_ptr = arr_ptr.add(j);
                if !is_less(&*gap.value, &*j_ptr) {
                    break;
                }

                ptr::copy_nonoverlapping(j_ptr, gap.pos, 1);
                gap.pos = j_ptr;
            }
            // `gap` gets dropped and thus copies `tmp` into the remaining gap in `v`.
        }
    }
}

/// Sort `v` assuming `v[..offset]` is already sorted.
pub(crate) fn insertion_sort_shift_left<T, F>(v: &mut [T], offset: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    if offset == 0 || offset > len {
        intrinsics::abort();
    }

    // Shift each element of the unsorted region v[i..] as far left as is needed to make v sorted.
    for i in offset..len {
        // SAFETY: we tested that `offset` must be at least 1, so this loop is only entered if len
        // >= 2.
        unsafe {
            insert_tail(&mut v[..=i], is_less);
        }
    }
}

#[inline(never)]
fn panic_on_ord_violation() -> ! {
    panic!("Ord violation");
}
