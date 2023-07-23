//! Inspired by Bitset Sort https://github.com/minjaehwang/bitsetsort. Modified to (TODO unroll swap
//! bitset blocks) and to do dynamic runtime feature detection.
//!
//! TODO explain why this is good. panic and ord safety, good unrolling, move efficiency, single
//! impl etc. With SIMD faster for types < u64 like i32.

use std::cmp;
use std::mem::MaybeUninit;
use std::ptr;

partition_impl!("hoare_block_ptr");

//  TODO explain

/// Scan elements `base_ptr[..block]` and build a bitset that has the corresponding bit toggled
/// depending on `is_swap_elem`.
///
/// SAFETY: The caller must ensure that `base_ptr[..block]` is valid to read. TODO
#[inline(always)]
unsafe fn fill_swap_ptr_block<T, F>(
    block_len: usize,
    base_ptr: *mut T,
    mut swap_ptr_block: *mut *mut T,
    is_swap_elem: &mut F,
) -> *mut *mut T
where
    F: FnMut(&T) -> bool,
{
    for i in 0..block_len {
        // SAFETY: See function safety comment.
        let elem_ptr = base_ptr.add(i);
        *swap_ptr_block = elem_ptr;
        let is_se = unsafe { is_swap_elem(&*elem_ptr) };
        swap_ptr_block = swap_ptr_block.add(is_se as usize);
    }

    swap_ptr_block
}

#[inline(always)]
unsafe fn fill_swap_ptr_block_down<T, F>(
    block_len: usize,
    base_ptr: *mut T,
    mut swap_ptr_block: *mut *mut T,
    is_swap_elem: &mut F,
) -> *mut *mut T
where
    F: FnMut(&T) -> bool,
{
    let base_ptr_up = base_ptr.add(block_len - 1);

    for i in 0..block_len {
        // SAFETY: See function safety comment.
        let elem_ptr = base_ptr_up.sub(i);
        *swap_ptr_block = elem_ptr;
        let is_se = unsafe { is_swap_elem(&*elem_ptr) };
        swap_ptr_block = swap_ptr_block.add(is_se as usize);
    }

    swap_ptr_block
}

/// Swap up to `BitsetStorageT::BITS` elements between `l_ptr[..BitsetStorageT::BITS]` and
/// `r_ptr[..BitsetStorageT::BITS]`. Where each bit set to one indicates that the element
/// should be swapped with one on other side. Both bitsets can have between 0 and
/// `BitsetStorageT::BITS` elements that need to be swapped. It will swap the number of elements
/// that can be swapped with a direct partner on the both sides. This may leave one of the sides
/// with un-swapped elements.
///
/// Returns the updated bitsets.
///
// SAFETY: The caller must ensure that `l_ptr[..swap_count]` and `r_ptr[..swap_count]` are
// valid to be written.
#[inline(always)]
unsafe fn swap_between_blocks<T>(
    l_swap_ptr_ptr: *const *mut T,
    r_swap_ptr_ptr: *const *mut T,
    count: usize,
) {
    // if count == 0 {
    //     return;
    // }

    // let mut left_elem_ptr = *l_swap_ptr_ptr;
    // let mut right_elem_ptr = *r_swap_ptr_ptr;

    // let tmp = ptr::read(left_elem_ptr);
    // ptr::copy_nonoverlapping(right_elem_ptr, left_elem_ptr, 1);

    // for i in 1..count {
    //     left_elem_ptr = *l_swap_ptr_ptr.add(i);
    //     ptr::copy_nonoverlapping(left_elem_ptr, right_elem_ptr, 1);

    //     right_elem_ptr = *r_swap_ptr_ptr.add(i);
    //     ptr::copy_nonoverlapping(right_elem_ptr, left_elem_ptr, 1);
    // }

    // ptr::copy_nonoverlapping(&tmp, right_elem_ptr, 1);
    // mem::forget(tmp);

    // TODO it seems the compiler can optimize this to re-use the temporary. Which makes this better
    // than the manual cyclic permutation?
    for i in 0..count {
        ptr::swap_nonoverlapping(*l_swap_ptr_ptr.add(i), *r_swap_ptr_ptr.add(i), 1);
    }
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // TODO explain more. Both AVX and NEON SIMD were analyzed for `u64`, the inner pivot comparison
    // loop should spend a bit less than a cycle per element doing the comparison and 1.5-2.5 cycles
    // if no SIMD is available. TODO cycles per swapped elements.

    const BLOCK: usize = 128;

    // lt == less than, ge == greater or equal

    let len = v.len();
    let arr_ptr = v.as_mut_ptr();

    // SAFETY: TODO
    unsafe {
        let mut l_ptr = arr_ptr;
        let mut r_ptr;

        let mut ge_ptr_buffer = MaybeUninit::<[*mut T; BLOCK]>::uninit();
        let mut ge_ptr_buffer_ptr = ge_ptr_buffer.as_mut_ptr() as *mut *mut T;
        let mut ge_ptr_buffer_ptr_base = ge_ptr_buffer_ptr;
        // let ge_end_ptr = ge_ptr_buffer_ptr.add(BLOCK);

        let mut lt_ptr_buffer = MaybeUninit::<[*mut T; BLOCK]>::uninit();
        let mut lt_ptr_buffer_ptr = lt_ptr_buffer.as_mut_ptr() as *mut *mut T;
        let mut lt_ptr_buffer_ptr_base = lt_ptr_buffer_ptr;
        // let lt_end_ptr = lt_ptr_buffer_ptr.add(BLOCK);

        let mut lt_count = 0;

        // It's crucial for reliable auto-vectorization that BLOCK always stays the same. Which
        // means we handle the rest of the input size separately later.
        if len >= (2 * BLOCK) {
            r_ptr = arr_ptr.add(len - BLOCK);

            // If the region we will look at during this loop iteration overlaps we are done.
            while l_ptr.add(BLOCK) <= r_ptr {
                // While interleaving left and right side access would be possible, experiments show
                // that on Zen3 this has significantly worse performance, and the CPU prefers
                // working on one region of memory followed by another.

                let is_refill_l = ge_ptr_buffer_ptr_base == ge_ptr_buffer_ptr;
                if is_refill_l {
                    ge_ptr_buffer_ptr_base = ge_ptr_buffer.as_mut_ptr() as *mut *mut T;
                    ge_ptr_buffer_ptr =
                        fill_swap_ptr_block(BLOCK, l_ptr, ge_ptr_buffer_ptr_base, &mut |elem| {
                            !is_less(elem, pivot)
                        });
                    lt_count += BLOCK - ge_ptr_buffer_ptr.sub_ptr(ge_ptr_buffer_ptr_base);
                }

                let is_refill_r = lt_ptr_buffer_ptr_base == lt_ptr_buffer_ptr;
                if is_refill_r {
                    lt_ptr_buffer_ptr_base = lt_ptr_buffer.as_mut_ptr() as *mut *mut T;
                    lt_ptr_buffer_ptr = fill_swap_ptr_block_down(
                        BLOCK,
                        r_ptr,
                        lt_ptr_buffer_ptr_base,
                        &mut |elem| is_less(elem, pivot),
                    );
                    lt_count += lt_ptr_buffer_ptr.sub_ptr(lt_ptr_buffer_ptr_base);
                }

                let swap_count = cmp::min(
                    ge_ptr_buffer_ptr.sub_ptr(ge_ptr_buffer_ptr_base),
                    lt_ptr_buffer_ptr.sub_ptr(lt_ptr_buffer_ptr_base),
                );

                swap_between_blocks(ge_ptr_buffer_ptr_base, lt_ptr_buffer_ptr_base, swap_count);

                ge_ptr_buffer_ptr_base = ge_ptr_buffer_ptr_base.add(swap_count);
                lt_ptr_buffer_ptr_base = lt_ptr_buffer_ptr_base.add(swap_count);

                l_ptr = l_ptr.add(is_refill_l as usize * BLOCK);
                r_ptr = r_ptr.sub(is_refill_r as usize * BLOCK); // TODO wrong wrapping
            }
        }

        // let remaining = r_ptr.sub_ptr(l_ptr);
        // dbg!(remaining);

        lt_count.saturating_sub(BLOCK) // FIXME
    }
}
