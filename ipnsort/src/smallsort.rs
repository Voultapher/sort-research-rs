use core::intrinsics;
use core::mem::{self, ManuallyDrop, MaybeUninit};
use core::ptr;

use crate::{has_efficient_in_place_swap, Freeze, GapGuard, IsTrue};

// Use a trait to focus code-gen on only the parts actually relevant for the type. Avoid generating
// LLVM-IR for the sorting-network and median-networks for types that don't qualify.
pub(crate) trait SmallSortImpl: Sized {
    const SMALL_SORT_THRESHOLD: usize;

    /// Sorts `v` using strategies optimized for small sizes.
    fn small_sort<F>(v: &mut [Self], is_less: &mut F)
    where
        F: FnMut(&Self, &Self) -> bool;
}

impl<T> SmallSortImpl for T {
    default const SMALL_SORT_THRESHOLD: usize = 20;

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
    default const SMALL_SORT_THRESHOLD: usize = 20;

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
    const SMALL_SORT_THRESHOLD: usize = 32;

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
    mut left_src: *const T,
    mut right_src: *const T,
    mut dst: *mut T,
    is_less: &mut F,
) -> (*const T, *const T, *mut T)
where
    F: FnMut(&T, &T) -> bool,
{
    // This is a branchless merge utility function.
    // The equivalent code with a branch would be:
    //
    // if !is_less(&*right_src, &*left_src) {
    //     ptr::copy_nonoverlapping(left_src, dst, 1);
    //     left_src = left_src.wrapping_add(1);
    // } else {
    //     ptr::copy_nonoverlapping(right_src, dst, 1);
    //     right_src = right_src.wrapping_add(1);
    // }
    // dst = dst.add(1);

    // SAFETY: The caller must guarantee that `left_src`, `right_src` are valid to read and
    // `dst` is valid to write, while not aliasing.
    unsafe {
        let is_l = !is_less(&*right_src, &*left_src);
        let src = if is_l { left_src } else { right_src };
        ptr::copy_nonoverlapping(src, dst, 1);
        right_src = right_src.wrapping_add(!is_l as usize);
        left_src = left_src.wrapping_add(is_l as usize);
        dst = dst.add(1);
    }

    (left_src, right_src, dst)
}

#[inline(always)]
unsafe fn merge_down<T, F>(
    mut left_src: *const T,
    mut right_src: *const T,
    mut dst: *mut T,
    is_less: &mut F,
) -> (*const T, *const T, *mut T)
where
    F: FnMut(&T, &T) -> bool,
{
    // This is a branchless merge utility function.
    // The equivalent code with a branch would be:
    //
    // if !is_less(&*right_src, &*left_src) {
    //     ptr::copy_nonoverlapping(right_src, dst, 1);
    //     right_src = right_src.wrapping_sub(1);
    // } else {
    //     ptr::copy_nonoverlapping(left_src, dst, 1);
    //     left_src = left_src.wrapping_sub(1);
    // }
    // dst = dst.sub(1);

    // SAFETY: The caller must guarantee that `left_src`, `right_src` are valid to read and
    // `dst` is valid to write, while not aliasing.
    unsafe {
        let is_l = !is_less(&*right_src, &*left_src);
        let src = if is_l { right_src } else { left_src };
        ptr::copy_nonoverlapping(src, dst, 1);
        right_src = right_src.wrapping_sub(is_l as usize);
        left_src = left_src.wrapping_sub(!is_l as usize);
        dst = dst.sub(1);
    }

    (left_src, right_src, dst)
}

/// Merge v assuming the len is even and v[..len / 2] and v[len / 2..] are sorted.
///
/// Original idea for bi-directional merging by Igor van den Hoven (quadsort), adapted to only use
/// merge up and down. In contrast to the original parity_merge function, it performs 2 writes
/// instead of 4 per iteration. Ord violation detection was added.
unsafe fn bi_directional_merge_even<T, F>(v: &[T], dst: *mut T, is_less: &mut F)
where
    T: crate::Freeze,
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: the caller must guarantee that `dst` is valid for v.len() writes.
    // Also `v.as_ptr` and `dst` must not alias.
    //
    // The caller must guarantee that T cannot modify itself inside is_less.
    // merge_up and merge_down read left and right pointers and potentially modify the stack value
    // they point to, if T has interior mutability. This may leave one or two potential writes to
    // the stack value un-observed when dst is copied onto of src.

    // It helps to visualize the merge:
    //
    // Initial:
    //
    //  |dst (in dst)
    //  |left               |right
    //  v                   v
    // [xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx]
    //                     ^                   ^
    //                     |left_rev           |right_rev
    //                                         |dst_rev (in dst)
    //
    // After:
    //
    //                      |dst (in dst)
    //        |left         |           |right
    //        v             v           v
    // [xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx]
    //       ^             ^           ^
    //       |left_rev     |           |right_rev
    //                     |dst_rev (in dst)
    //
    //
    // Note, the pointers that have been written, are now one past where they were read and
    // copied. written == incremented or decremented + copy to dst.

    let len = v.len();
    let src = v.as_ptr();
    unsafe { intrinsics::assume(len > 0) };

    let len_div_2 = len / 2;

    // SAFETY: No matter what the result of the user-provided comparison function is, all 4 read
    // pointers will always be in-bounds. Writing `dst` and `dst_rev` will always be in
    // bounds if the caller guarantees that `dst` is valid for `v.len()` writes.
    unsafe {
        let mut left = src;
        let mut right = src.wrapping_add(len_div_2);
        let mut dst = dst;

        let mut left_rev = src.wrapping_add(len_div_2 - 1);
        let mut right_rev = src.wrapping_add(len - 1);
        let mut dst_rev = dst.wrapping_add(len - 1);

        for _ in 0..len_div_2 {
            (left, right, dst) = merge_up(left, right, dst, is_less);
            (left_rev, right_rev, dst_rev) = merge_down(left_rev, right_rev, dst_rev, is_less);
        }

        let left_diff = (left as usize).wrapping_sub(left_rev as usize);
        let right_diff = (right as usize).wrapping_sub(right_rev as usize);

        if !(left_diff == mem::size_of::<T>() && right_diff == mem::size_of::<T>()) {
            panic_on_ord_violation();
        }
    }
}

#[inline(always)]
pub unsafe fn branchless_swap<T>(left: *mut T, right: *mut T, should_swap: bool) {
    // SAFETY: the caller must guarantee that `left` and `right` are valid for writes
    // and properly aligned, and part of the same allocation, and do not alias.

    // This is a branchless version of swap if.
    // The equivalent code with a branch would be:
    //
    // if should_swap {
    //     ptr::swap(left, right, 1);
    // }

    // Give ourselves some scratch space to work with.
    // We do not have to worry about drops: `MaybeUninit` does nothing when dropped.

    // The goal is to generate cmov instructions here.
    let left_swap = if should_swap { right } else { left };
    let right_swap = if should_swap { left } else { right };

    let right_swap_tmp = ManuallyDrop::new(ptr::read(right_swap));

    ptr::copy(left_swap, left, 1);
    ptr::copy_nonoverlapping(&*right_swap_tmp, right, 1);
}

/// Swap two values in the slice pointed to by `v_base` at the position `a_pos` and `b_pos` if the
/// value at position `b_pos` is less than the one at position `a_pos`.
pub unsafe fn swap_if_less<T, F>(v_base: *mut T, a_pos: usize, b_pos: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: the caller must guarantee that `a` and `b` each added to `v_base` yield valid
    // pointers into `v_base`, and are properly aligned, and part of the same allocation.

    let v_a = v_base.add(a_pos);
    let v_b = v_base.add(b_pos);

    // PANIC SAFETY: if is_less panics, no scratch memory was created and the slice should still be
    // in a well defined state, without duplicates.

    // Important to only swap if it is more and not if it is equal. is_less should return false for
    // equal, so we don't swap.
    let should_swap = is_less(&*v_b, &*v_a);
    branchless_swap(v_a, v_b, should_swap);
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

    let v_base = v.as_mut_ptr();

    // Optimal sorting network see:
    // https://bertdobbelaere.github.io/sorting_networks.html.

    // SAFETY: We checked the len.
    unsafe {
        swap_if_less(v_base, 0, 3, is_less);
        swap_if_less(v_base, 1, 7, is_less);
        swap_if_less(v_base, 2, 5, is_less);
        swap_if_less(v_base, 4, 8, is_less);
        swap_if_less(v_base, 0, 7, is_less);
        swap_if_less(v_base, 2, 4, is_less);
        swap_if_less(v_base, 3, 8, is_less);
        swap_if_less(v_base, 5, 6, is_less);
        swap_if_less(v_base, 0, 2, is_less);
        swap_if_less(v_base, 1, 3, is_less);
        swap_if_less(v_base, 4, 5, is_less);
        swap_if_less(v_base, 7, 8, is_less);
        swap_if_less(v_base, 1, 4, is_less);
        swap_if_less(v_base, 3, 6, is_less);
        swap_if_less(v_base, 5, 7, is_less);
        swap_if_less(v_base, 0, 1, is_less);
        swap_if_less(v_base, 2, 4, is_less);
        swap_if_less(v_base, 3, 5, is_less);
        swap_if_less(v_base, 6, 8, is_less);
        swap_if_less(v_base, 2, 3, is_less);
        swap_if_less(v_base, 4, 5, is_less);
        swap_if_less(v_base, 6, 7, is_less);
        swap_if_less(v_base, 1, 2, is_less);
        swap_if_less(v_base, 3, 4, is_less);
        swap_if_less(v_base, 5, 6, is_less);
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

    let v_base = v.as_mut_ptr();

    // Optimal sorting network see:
    // https://bertdobbelaere.github.io/sorting_networks.html.

    // SAFETY: We checked the len.
    unsafe {
        swap_if_less(v_base, 0, 12, is_less);
        swap_if_less(v_base, 1, 10, is_less);
        swap_if_less(v_base, 2, 9, is_less);
        swap_if_less(v_base, 3, 7, is_less);
        swap_if_less(v_base, 5, 11, is_less);
        swap_if_less(v_base, 6, 8, is_less);
        swap_if_less(v_base, 1, 6, is_less);
        swap_if_less(v_base, 2, 3, is_less);
        swap_if_less(v_base, 4, 11, is_less);
        swap_if_less(v_base, 7, 9, is_less);
        swap_if_less(v_base, 8, 10, is_less);
        swap_if_less(v_base, 0, 4, is_less);
        swap_if_less(v_base, 1, 2, is_less);
        swap_if_less(v_base, 3, 6, is_less);
        swap_if_less(v_base, 7, 8, is_less);
        swap_if_less(v_base, 9, 10, is_less);
        swap_if_less(v_base, 11, 12, is_less);
        swap_if_less(v_base, 4, 6, is_less);
        swap_if_less(v_base, 5, 9, is_less);
        swap_if_less(v_base, 8, 11, is_less);
        swap_if_less(v_base, 10, 12, is_less);
        swap_if_less(v_base, 0, 5, is_less);
        swap_if_less(v_base, 3, 8, is_less);
        swap_if_less(v_base, 4, 7, is_less);
        swap_if_less(v_base, 6, 11, is_less);
        swap_if_less(v_base, 9, 10, is_less);
        swap_if_less(v_base, 0, 1, is_less);
        swap_if_less(v_base, 2, 5, is_less);
        swap_if_less(v_base, 6, 9, is_less);
        swap_if_less(v_base, 7, 8, is_less);
        swap_if_less(v_base, 10, 11, is_less);
        swap_if_less(v_base, 1, 3, is_less);
        swap_if_less(v_base, 2, 4, is_less);
        swap_if_less(v_base, 5, 6, is_less);
        swap_if_less(v_base, 9, 10, is_less);
        swap_if_less(v_base, 1, 2, is_less);
        swap_if_less(v_base, 3, 4, is_less);
        swap_if_less(v_base, 5, 7, is_less);
        swap_if_less(v_base, 6, 8, is_less);
        swap_if_less(v_base, 2, 3, is_less);
        swap_if_less(v_base, 4, 5, is_less);
        swap_if_less(v_base, 6, 7, is_less);
        swap_if_less(v_base, 8, 9, is_less);
        swap_if_less(v_base, 3, 4, is_less);
        swap_if_less(v_base, 5, 6, is_less);
    }
}

fn sort18_plus<T, F>(v: &mut [T], is_less: &mut F) -> usize
where
    T: Freeze,
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    const MAX_BRANCHLESS_SMALL_SORT: usize = i32::SMALL_SORT_THRESHOLD;

    if len < 18 || len > MAX_BRANCHLESS_SMALL_SORT {
        intrinsics::abort();
    }

    // This should optimize to a shift right.
    let even_len = len - (len % 2);
    let len_div_2 = even_len / 2;

    let presorted_len = if len < 26 {
        sort9_optimal(&mut v[0..9], is_less);
        sort9_optimal(&mut v[len_div_2..(len_div_2 + 9)], is_less);

        9
    } else {
        sort13_optimal(&mut v[0..13], is_less);
        sort13_optimal(&mut v[len_div_2..(len_div_2 + 13)], is_less);

        13
    };

    insertion_sort_shift_left(&mut v[0..len_div_2], presorted_len, is_less);
    insertion_sort_shift_left(&mut v[len_div_2..even_len], presorted_len, is_less);

    let mut scratch = MaybeUninit::<[T; MAX_BRANCHLESS_SMALL_SORT]>::uninit();
    let scratch_base = scratch.as_mut_ptr() as *mut T;

    // SAFETY: We checked that T is Freeze and thus observation safe.
    // Should is_less panic v was not modified in parity_merge and retains it's original input.
    // scratch and v must not alias and scratch has v.len() space.
    unsafe {
        bi_directional_merge_even(&mut v[..even_len], scratch_base, is_less);
        ptr::copy_nonoverlapping(scratch_base, v.as_mut_ptr(), even_len);
    }

    even_len
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
    if len >= 2 {
        let mut end = 1;
        if len >= 18 {
            end = sort18_plus(v, is_less);
        } else if len >= 13 {
            sort13_optimal(&mut v[0..13], is_less);
            end = 13;
        } else if len >= 9 {
            sort9_optimal(&mut v[0..9], is_less);
            end = 9;
        }

        insertion_sort_shift_left(v, end, is_less);
    }
}

fn small_sort_general<T, F>(v: &mut [T], is_less: &mut F)
where
    T: Freeze,
    F: FnMut(&T, &T) -> bool,
{
    // This implementation is tuned to be efficient for various types that are larger than u64.

    let len = v.len();

    if len >= 2 {
        const SCRATCH_LEN: usize = String::SMALL_SORT_THRESHOLD + 16;
        let mut scratch = MaybeUninit::<[T; SCRATCH_LEN]>::uninit();

        if SCRATCH_LEN < (T::SMALL_SORT_THRESHOLD + 16) {
            intrinsics::abort();
        }

        let v_base = v.as_mut_ptr();

        let offset = if len >= 8 {
            let len_div_2 = len / 2;

            // SAFETY: TODO
            unsafe {
                let scratch_base = scratch.as_mut_ptr() as *mut T;

                let presorted_len = if len >= 16 {
                    // SAFETY: scratch_base is valid and has enough space.
                    sort8_stable(
                        v_base,
                        scratch_base.add(T::SMALL_SORT_THRESHOLD),
                        scratch_base,
                        is_less,
                    );

                    sort8_stable(
                        v_base.add(len_div_2),
                        scratch_base.add(T::SMALL_SORT_THRESHOLD + 8),
                        scratch_base.add(len_div_2),
                        is_less,
                    );

                    8
                } else {
                    // SAFETY: scratch_base is valid and has enough space.
                    sort4_stable(v_base, scratch_base, is_less);
                    sort4_stable(v_base.add(len_div_2), scratch_base.add(len_div_2), is_less);

                    4
                };

                for offset in [0, len_div_2] {
                    let src = scratch_base.add(offset);
                    let dst = v_base.add(offset);

                    for i in presorted_len..len_div_2 {
                        ptr::copy_nonoverlapping(dst.add(i), src.add(i), 1);
                        let scratch_slice = &mut *ptr::slice_from_raw_parts_mut(src, i + 1);
                        insert_tail(scratch_slice, is_less);
                    }
                }

                let even_len = len - (len % 2);

                // See comment in `DropGuard::drop`.
                let drop_guard = DropGuard {
                    src: scratch_base,
                    dst: v_base,
                    len: even_len,
                };

                // It's faster to merge directly into `v` and copy over the 'safe' elements of
                // `scratch` into v only if there was a panic. This technique is similar to
                // ping-pong merging.
                bi_directional_merge_even(
                    &*ptr::slice_from_raw_parts(drop_guard.src, drop_guard.len),
                    drop_guard.dst,
                    is_less,
                );
                mem::forget(drop_guard);

                even_len
            }
        } else {
            1
        };

        insertion_sort_shift_left(v, offset, is_less);
    }

    struct DropGuard<T> {
        src: *mut T,
        dst: *mut T,
        len: usize,
    }

    impl<T> Drop for DropGuard<T> {
        fn drop(&mut self) {
            // SAFETY: `src` must hold the original `len` elements of `v` in any order. And dst
            // must be valid to write `len` elements.
            unsafe {
                ptr::copy_nonoverlapping(self.src, self.dst, self.len);
            }
        }
    }
}

/// SAFETY: The caller MUST guarantee that `v_base` is valid for 4 reads and `dest_ptr` is valid
/// for 4 writes. The result will be stored in `dst[0..4]`.
pub unsafe fn sort4_stable<T, F>(v_base: *const T, dst: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // By limiting select to picking pointers, we are guaranteed good cmov code-gen regardless of
    // type T layout. Further this only does 5 instead of 6 comparisons compared to a stable
    // transposition 4 element sorting-network. Also by only operating on pointers, we get optimal
    // element copy usage. Doing exactly 1 copy per element.

    // let v_base = v.as_ptr();

    unsafe {
        // Stably create two pairs a <= b and c <= d.
        let c1 = is_less(&*v_base.add(1), &*v_base);
        let c2 = is_less(&*v_base.add(3), &*v_base.add(2));
        let a = v_base.add(c1 as usize);
        let b = v_base.add(!c1 as usize);
        let c = v_base.add(2 + c2 as usize);
        let d = v_base.add(2 + (!c2 as usize));

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

        ptr::copy_nonoverlapping(min, dst, 1);
        ptr::copy_nonoverlapping(lo, dst.add(1), 1);
        ptr::copy_nonoverlapping(hi, dst.add(2), 1);
        ptr::copy_nonoverlapping(max, dst.add(3), 1);
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

/// SAFETY: The caller MUST guarantee that `v_base` is valid for 8 reads and writes, `scratch_base`
/// and `dst` MUST be valid for 8 writes. The result will be stored in `dst[0..8]`.
#[inline(never)]
unsafe fn sort8_stable<T, F>(v_base: *mut T, scratch_base: *mut T, dst: *mut T, is_less: &mut F)
where
    T: crate::Freeze,
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: The caller must guarantee that scratch_base is valid for 8 writes, and that v_base is
    // valid for 8 reads.
    unsafe {
        sort4_stable(v_base, scratch_base, is_less);
        sort4_stable(v_base.add(4), scratch_base.add(4), is_less);
    }

    // SAFETY: TODO
    unsafe {
        bi_directional_merge_even(&*ptr::slice_from_raw_parts(scratch_base, 8), dst, is_less);
    }
}

// --- Insertion sort ---

/// Inserts `v[v.len() - 1]` into pre-sorted sequence `v[..v.len() - 1]` so that whole `v[..]`
/// becomes sorted.
fn insert_tail<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    if v.len() < 2 {
        intrinsics::abort();
    }

    let v_base = v.as_mut_ptr();
    let i = v.len() - 1;

    // SAFETY: We checked that `v.len()` is at least 2.
    unsafe {
        // See insert_head which talks about why this approach is beneficial.
        let v_i = v_base.add(i);

        // It's important that we use v_i here. If this check is positive and we continue,
        // We want to make sure that no other copy of the value was seen by is_less.
        // Otherwise we would have to copy it back.
        if is_less(&*v_i, &*v_i.sub(1)) {
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
            let mut gap = GapGuard {
                pos: v_i.sub(1),
                value: ManuallyDrop::new(ptr::read(v_i)),
            };
            ptr::copy_nonoverlapping(gap.pos, v_i, 1);

            // SAFETY: We know i is at least 1.
            for j in (0..(i - 1)).rev() {
                let v_j = v_base.add(j);
                if !is_less(&*gap.value, &*v_j) {
                    break;
                }

                ptr::copy_nonoverlapping(v_j, gap.pos, 1);
                gap.pos = v_j;
            }
            // `gap` gets dropped and thus copies `tmp` into the remaining gap in `v`.
        }
    }
}

/// Sort `v` assuming `v[..offset]` is already sorted.
pub fn insertion_sort_shift_left<T, F>(v: &mut [T], offset: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    // This would be a logic bug in other code.
    debug_assert!(offset != 0 && offset <= len);

    // Shift each element of the unsorted region v[i..] as far left as is needed to make v sorted.
    for i in offset..len {
        insert_tail(&mut v[..=i], is_less);
    }
}

#[inline(never)]
fn panic_on_ord_violation() -> ! {
    panic!("Ord violation");
}
