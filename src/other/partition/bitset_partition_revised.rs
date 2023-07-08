//! Inspired by Bitset Sort https://github.com/minjaehwang/bitsetsort. Modified to (TODO unroll swap
//! bitset blocks) and to do dynamic runtime feature detection.
//!
//! TODO explain why this is good. panic and ord safety, good unrolling, move efficiency, single
//! impl etc. With SIMD faster for types < u64 like i32. TODO talk about if already partitioned
//! perf relevant for common values filtered out via pdqsort logic.

use std::cmp;
use std::mem::{self, MaybeUninit};
use std::ptr;

partition_impl!("bitset_partition_revised");

// Using 32 bits as bitset and with that as block-size has various benefits. It nicely unrolls the
// inner pivot comparison loop into a single block of SIMD instructions and it doesn't tempt the
// prefetcher into fetching too much info on the right side. This is with `u64` as the largest type
// expected to greatly benefit from vectorization.
type BitsetStorageT = u32;

/// Scan elements `base_ptr[..block_len]` and build a bitset that has the corresponding bit toggled
/// depending on `is_swap_elem`.
///
/// SAFETY: The caller must ensure that `base_ptr[..block_len]` is valid to read.
#[inline(always)]
unsafe fn fill_bitset<T>(
    block_len: usize,
    base_ptr: *const T,
    is_swap_elem: &mut impl FnMut(&T) -> bool,
) -> BitsetStorageT {
    debug_assert!(block_len <= BitsetStorageT::BITS as usize);

    let mut bitset: BitsetStorageT = 0;

    for i in 0..block_len {
        // SAFETY: See function safety comment.
        let is_se = unsafe { is_swap_elem(&*base_ptr.add(i)) };
        bitset |= (is_se as BitsetStorageT) << (i as u32);
    }

    bitset
}

#[inline(always)]
fn clear_lowest_bit(x: BitsetStorageT) -> BitsetStorageT {
    let mask = x - 1;

    x & mask
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
// SAFETY: The caller must ensure that each `l_ptr[swap_pos]` and `r_ptr[swap_pos]` are valid to be
// written. Where `swap_pos` is a number between `0` and `BitsetStorageT::BITS` depending on the
// values in `l_bitmap` and `r_bitmap`.
#[inline(always)]
unsafe fn swap_between_blocks<T>(
    l_ptr: *mut T,
    r_ptr: *mut T,
    mut l_bitmap: BitsetStorageT,
    mut r_bitmap: BitsetStorageT,
) -> (BitsetStorageT, BitsetStorageT) {
    // Instead of swapping one pair at the time, it is more efficient to perform a cyclic
    // permutation. This is not strictly equivalent to swapping, but produces a similar
    // result using fewer memory operations.
    //
    // Example cyclic permutation to swap A,B,C,D with W,X,Y,Z
    //
    // A -> TMP
    // Z -> A   | Z,B,C,D ___ W,X,Y,Z
    //
    // Loop iter 1
    // B -> Z   | Z,B,C,D ___ W,X,Y,B
    // Y -> B   | Z,Y,C,D ___ W,X,Y,B
    //
    // Loop iter 2
    // C -> Y   | Z,Y,C,D ___ W,X,C,B
    // X -> C   | Z,Y,X,D ___ W,X,C,B
    //
    // Loop iter 3
    // D -> X   | Z,Y,X,D ___ W,D,C,B
    // W -> D   | Z,Y,X,W ___ W,D,C,B
    //
    // TMP -> W | Z,Y,X,W ___ A,D,C,B
    if l_bitmap > 0 && r_bitmap > 0 {
        // SAFETY: Based on the caller provided safety guarantees we are safe to swap each element
        // between the left and right block. The following code is guaranteed panic-free which
        // ensure the temporary we crate is panic- and observation-safe.
        unsafe {
            let left = |l_bitmap: &mut BitsetStorageT| {
                let l_idx = l_bitmap.trailing_zeros() as usize;
                *l_bitmap = clear_lowest_bit(*l_bitmap);
                l_ptr.add(l_idx)
            };

            let right = |r_bitmap: &mut BitsetStorageT| {
                let r_idx = r_bitmap.trailing_zeros() as usize;
                *r_bitmap = clear_lowest_bit(*r_bitmap);
                r_ptr.add(r_idx)
            };

            let mut left_ptr = left(&mut l_bitmap);
            let mut right_ptr = right(&mut r_bitmap);

            let tmp = ptr::read(left_ptr);
            ptr::copy_nonoverlapping(right_ptr, left_ptr, 1);

            // Surprisingly doing popcnt + for in 0..swap_count is a lot slower.
            while l_bitmap > 0 && r_bitmap > 0 {
                left_ptr = left(&mut l_bitmap);
                ptr::copy_nonoverlapping(left_ptr, right_ptr, 1);
                right_ptr = right(&mut r_bitmap);
                ptr::copy_nonoverlapping(right_ptr, left_ptr, 1);
            }

            ptr::copy_nonoverlapping(&tmp, right_ptr, 1);
            mem::forget(tmp);
        }
    }

    (l_bitmap, r_bitmap)
}

// #[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx")]
unsafe fn partition_impl<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // TODO explain more. Both AVX and NEON SIMD were analyzed for `u64` and `i32` element types,
    // the inner pivot comparison loop should spend a bit less than a cycle per element doing the
    // comparison and 1.5-2.5 cycles if no SIMD is available. TODO cycles per swapped elements.
    const BLOCK: usize = 1;
    // assert!((BLOCK * 2) <= 2usize.pow(u8::BITS));

    // lt == less than, ge == greater or equal

    let len = v.len();
    let arr_ptr = v.as_mut_ptr();

    if core::intrinsics::unlikely(len < 2) {
        return len;
    }

    // SAFETY: TODO
    unsafe {
        let mut l_ptr = arr_ptr;
        let mut r_end_ptr = arr_ptr.add(len);

        let mut l_bitmap: BitsetStorageT = 0; // aka ge_bitmap
        let mut r_bitmap: BitsetStorageT = 0; // aka lt_bitmap

        // It's crucial for reliable auto-vectorization that BLOCK always stays the same. Which
        // means we handle the rest of the input size separately later.
        if len >= (2 * BLOCK) {
            let mut r_ptr = arr_ptr.add(len - BLOCK);

            // If the region we will look at during this loop iteration overlaps we are done.
            while l_ptr.add(BLOCK) <= r_ptr {
                // loop {
                // While interleaving left and right side access would be possible, experiments show
                // that on Zen3 this has significantly worse performance, and the CPU prefers working on
                // one region of memory followed by another.
                if l_bitmap == 0 {
                    l_bitmap = fill_bitset(BLOCK, l_ptr, &mut |elem| !is_less(elem, pivot));
                }

                if r_bitmap == 0 {
                    r_bitmap = fill_bitset(BLOCK, r_ptr, &mut |elem| is_less(elem, pivot));
                }

                (l_bitmap, r_bitmap) = swap_between_blocks(l_ptr, r_ptr, l_bitmap, r_bitmap);

                l_ptr = l_ptr.add((l_bitmap == 0) as usize * BLOCK);
                r_ptr = r_ptr.sub((r_bitmap == 0) as usize * BLOCK);
            }

            // Switch to other conceptual model where r_end_ptr is the end of the right side instead
            // of the start.
            r_end_ptr = r_ptr.add(BLOCK);

            // Take care of the remaining elements in the unfinished bitmap if any.

            // It would be a logic bug if somehow swap_between_blocks left both blocks with
            // remaining elements.
            debug_assert!(!(l_bitmap != 0 && r_bitmap != 0));
        }

        // The following is optimized differently than the main block loop above. It tries to be
        // fast and binary efficient based on the fact that the remaining window is small and
        // contiguous. This is also crucial for perf of the relatively more common calls to
        // partition with smaller slices that have sizes which exceed the small-sort.

        // #[repr(align(64))]
        // struct

        // TODO explain
        let mut ge_idx_buffer = MaybeUninit::<[u8; BLOCK * 2]>::uninit();
        let mut ge_idx_ptr = ge_idx_buffer.as_mut_ptr() as *mut u8;

        let mut lt_idx_buffer = MaybeUninit::<[u8; BLOCK * 2]>::uninit();
        let mut lt_idx_ptr = lt_idx_buffer.as_mut_ptr() as *mut u8;

        let remainder = r_end_ptr.sub_ptr(l_ptr);
        // dbg!(remainder);
        debug_assert!(remainder < (BLOCK * 2));
        return l_ptr.sub_ptr(arr_ptr);

        macro_rules! set_idx_ptrs(
            ($i:expr) => {
                *lt_idx_ptr = $i;
                *ge_idx_ptr = $i;
                let is_lt = is_less(&*l_ptr.add($i as usize), pivot);
                lt_idx_ptr = lt_idx_ptr.add(is_lt as usize);
                ge_idx_ptr = ge_idx_ptr.add(!is_lt as usize);
            }
        );

        // Manually unrolled because on Arm LLVM doesn't do so and that's terrible for perf.
        let mut i: u8 = 0;

        // if l_bitmap != 0 {
        //     i =
        // }

        let end = remainder as u8 + i;

        while (i + 1) < end {
            set_idx_ptrs!(i);
            set_idx_ptrs!(i + 1);

            i += 2;
        }

        if (remainder % 2) != 0 {
            set_idx_ptrs!(i);
        }

        let ge_idx_base_ptr = ge_idx_buffer.as_ptr() as *const u8;
        let ge_count = ge_idx_ptr.sub_ptr(ge_idx_base_ptr);

        let lt_idx_base_ptr = lt_idx_buffer.as_ptr() as *const u8;
        let lt_count = lt_idx_ptr.sub_ptr(lt_idx_base_ptr);

        let swap_count = cmp::min(ge_count, lt_count);

        // println!(
        //     "\nge_idx_buffer: {:?}",
        //     &*ptr::slice_from_raw_parts(ge_idx_buffer.as_ptr() as *const u8, ge_count)
        // );
        // println!(
        //     "lt_idx_buffer: {:?}",
        //     &*ptr::slice_from_raw_parts(lt_idx_buffer.as_ptr() as *const u8, lt_count)
        // );

        // type DebugT = i32;
        lt_idx_ptr = lt_idx_ptr.wrapping_sub(1);

        // TODO benchmark cyclic permutation.
        for i in 0..swap_count {
            let l_ge_idx = *ge_idx_base_ptr.add(i) as usize;
            if l_ge_idx >= lt_count {
                break;
            }

            let r_lt_idx = *lt_idx_ptr.sub(i) as usize;

            // println!(
            //     "swapping {} <-> {} | idx {l_ge_idx} <-> {r_lt_idx}",
            //     *(l_ptr.add(l_ge_idx) as *const DebugT),
            //     *(l_ptr.add(r_lt_idx) as *const DebugT),
            // );

            ptr::swap_nonoverlapping(l_ptr.add(l_ge_idx), l_ptr.add(r_lt_idx), 1);
        }

        // let remaining = r_ptr.sub_ptr(l_ptr);
        // dbg!(remaining);

        l_ptr.sub_ptr(arr_ptr) + lt_count
    }
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: features have to be present.
    unsafe { partition_impl(v, pivot, is_less) }
}
