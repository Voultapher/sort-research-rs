#![allow(unused)]

use core::arch::x86_64;
use core::cmp;
use core::intrinsics;
use core::mem::{self, MaybeUninit};
use core::ptr;

partition_impl!("avx2");

#[inline]
#[cfg(target_arch = "x86_64")]
unsafe fn update_offsets_ptr(
    partiton_mask: u8,
    mut offsets_ptr: *mut u8,
    index_offset: usize,
) -> *mut u8 {
    // Relevant indices that needs to be written based on 4 bit mask.
    // Mask[0] == offset 0
    // Mask[1] == offset 1
    // Mask[2] == offset 2
    // Mask[3] == offset 3
    const INDEX_LOOKUP_MAP: [(u64, u8); 256] = [
        (0x0000000000000000, 0),
        (0x0000000000000000, 1),
        (0x0000000000000001, 1),
        (0x0000000000000100, 2),
        (0x0000000000000002, 1),
        (0x0000000000000200, 2),
        (0x0000000000000201, 2),
        (0x0000000000020100, 3),
        (0x0000000000000003, 1),
        (0x0000000000000300, 2),
        (0x0000000000000301, 2),
        (0x0000000000030100, 3),
        (0x0000000000000302, 2),
        (0x0000000000030200, 3),
        (0x0000000000030201, 3),
        (0x0000000003020100, 4),
        (0x0000000000000004, 1),
        (0x0000000000000400, 2),
        (0x0000000000000401, 2),
        (0x0000000000040100, 3),
        (0x0000000000000402, 2),
        (0x0000000000040200, 3),
        (0x0000000000040201, 3),
        (0x0000000004020100, 4),
        (0x0000000000000403, 2),
        (0x0000000000040300, 3),
        (0x0000000000040301, 3),
        (0x0000000004030100, 4),
        (0x0000000000040302, 3),
        (0x0000000004030200, 4),
        (0x0000000004030201, 4),
        (0x0000000403020100, 5),
        (0x0000000000000005, 1),
        (0x0000000000000500, 2),
        (0x0000000000000501, 2),
        (0x0000000000050100, 3),
        (0x0000000000000502, 2),
        (0x0000000000050200, 3),
        (0x0000000000050201, 3),
        (0x0000000005020100, 4),
        (0x0000000000000503, 2),
        (0x0000000000050300, 3),
        (0x0000000000050301, 3),
        (0x0000000005030100, 4),
        (0x0000000000050302, 3),
        (0x0000000005030200, 4),
        (0x0000000005030201, 4),
        (0x0000000503020100, 5),
        (0x0000000000000504, 2),
        (0x0000000000050400, 3),
        (0x0000000000050401, 3),
        (0x0000000005040100, 4),
        (0x0000000000050402, 3),
        (0x0000000005040200, 4),
        (0x0000000005040201, 4),
        (0x0000000504020100, 5),
        (0x0000000000050403, 3),
        (0x0000000005040300, 4),
        (0x0000000005040301, 4),
        (0x0000000504030100, 5),
        (0x0000000005040302, 4),
        (0x0000000504030200, 5),
        (0x0000000504030201, 5),
        (0x0000050403020100, 6),
        (0x0000000000000006, 1),
        (0x0000000000000600, 2),
        (0x0000000000000601, 2),
        (0x0000000000060100, 3),
        (0x0000000000000602, 2),
        (0x0000000000060200, 3),
        (0x0000000000060201, 3),
        (0x0000000006020100, 4),
        (0x0000000000000603, 2),
        (0x0000000000060300, 3),
        (0x0000000000060301, 3),
        (0x0000000006030100, 4),
        (0x0000000000060302, 3),
        (0x0000000006030200, 4),
        (0x0000000006030201, 4),
        (0x0000000603020100, 5),
        (0x0000000000000604, 2),
        (0x0000000000060400, 3),
        (0x0000000000060401, 3),
        (0x0000000006040100, 4),
        (0x0000000000060402, 3),
        (0x0000000006040200, 4),
        (0x0000000006040201, 4),
        (0x0000000604020100, 5),
        (0x0000000000060403, 3),
        (0x0000000006040300, 4),
        (0x0000000006040301, 4),
        (0x0000000604030100, 5),
        (0x0000000006040302, 4),
        (0x0000000604030200, 5),
        (0x0000000604030201, 5),
        (0x0000060403020100, 6),
        (0x0000000000000605, 2),
        (0x0000000000060500, 3),
        (0x0000000000060501, 3),
        (0x0000000006050100, 4),
        (0x0000000000060502, 3),
        (0x0000000006050200, 4),
        (0x0000000006050201, 4),
        (0x0000000605020100, 5),
        (0x0000000000060503, 3),
        (0x0000000006050300, 4),
        (0x0000000006050301, 4),
        (0x0000000605030100, 5),
        (0x0000000006050302, 4),
        (0x0000000605030200, 5),
        (0x0000000605030201, 5),
        (0x0000060503020100, 6),
        (0x0000000000060504, 3),
        (0x0000000006050400, 4),
        (0x0000000006050401, 4),
        (0x0000000605040100, 5),
        (0x0000000006050402, 4),
        (0x0000000605040200, 5),
        (0x0000000605040201, 5),
        (0x0000060504020100, 6),
        (0x0000000006050403, 4),
        (0x0000000605040300, 5),
        (0x0000000605040301, 5),
        (0x0000060504030100, 6),
        (0x0000000605040302, 5),
        (0x0000060504030200, 6),
        (0x0000060504030201, 6),
        (0x0006050403020100, 7),
        (0x0000000000000007, 1),
        (0x0000000000000700, 2),
        (0x0000000000000701, 2),
        (0x0000000000070100, 3),
        (0x0000000000000702, 2),
        (0x0000000000070200, 3),
        (0x0000000000070201, 3),
        (0x0000000007020100, 4),
        (0x0000000000000703, 2),
        (0x0000000000070300, 3),
        (0x0000000000070301, 3),
        (0x0000000007030100, 4),
        (0x0000000000070302, 3),
        (0x0000000007030200, 4),
        (0x0000000007030201, 4),
        (0x0000000703020100, 5),
        (0x0000000000000704, 2),
        (0x0000000000070400, 3),
        (0x0000000000070401, 3),
        (0x0000000007040100, 4),
        (0x0000000000070402, 3),
        (0x0000000007040200, 4),
        (0x0000000007040201, 4),
        (0x0000000704020100, 5),
        (0x0000000000070403, 3),
        (0x0000000007040300, 4),
        (0x0000000007040301, 4),
        (0x0000000704030100, 5),
        (0x0000000007040302, 4),
        (0x0000000704030200, 5),
        (0x0000000704030201, 5),
        (0x0000070403020100, 6),
        (0x0000000000000705, 2),
        (0x0000000000070500, 3),
        (0x0000000000070501, 3),
        (0x0000000007050100, 4),
        (0x0000000000070502, 3),
        (0x0000000007050200, 4),
        (0x0000000007050201, 4),
        (0x0000000705020100, 5),
        (0x0000000000070503, 3),
        (0x0000000007050300, 4),
        (0x0000000007050301, 4),
        (0x0000000705030100, 5),
        (0x0000000007050302, 4),
        (0x0000000705030200, 5),
        (0x0000000705030201, 5),
        (0x0000070503020100, 6),
        (0x0000000000070504, 3),
        (0x0000000007050400, 4),
        (0x0000000007050401, 4),
        (0x0000000705040100, 5),
        (0x0000000007050402, 4),
        (0x0000000705040200, 5),
        (0x0000000705040201, 5),
        (0x0000070504020100, 6),
        (0x0000000007050403, 4),
        (0x0000000705040300, 5),
        (0x0000000705040301, 5),
        (0x0000070504030100, 6),
        (0x0000000705040302, 5),
        (0x0000070504030200, 6),
        (0x0000070504030201, 6),
        (0x0007050403020100, 7),
        (0x0000000000000706, 2),
        (0x0000000000070600, 3),
        (0x0000000000070601, 3),
        (0x0000000007060100, 4),
        (0x0000000000070602, 3),
        (0x0000000007060200, 4),
        (0x0000000007060201, 4),
        (0x0000000706020100, 5),
        (0x0000000000070603, 3),
        (0x0000000007060300, 4),
        (0x0000000007060301, 4),
        (0x0000000706030100, 5),
        (0x0000000007060302, 4),
        (0x0000000706030200, 5),
        (0x0000000706030201, 5),
        (0x0000070603020100, 6),
        (0x0000000000070604, 3),
        (0x0000000007060400, 4),
        (0x0000000007060401, 4),
        (0x0000000706040100, 5),
        (0x0000000007060402, 4),
        (0x0000000706040200, 5),
        (0x0000000706040201, 5),
        (0x0000070604020100, 6),
        (0x0000000007060403, 4),
        (0x0000000706040300, 5),
        (0x0000000706040301, 5),
        (0x0000070604030100, 6),
        (0x0000000706040302, 5),
        (0x0000070604030200, 6),
        (0x0000070604030201, 6),
        (0x0007060403020100, 7),
        (0x0000000000070605, 3),
        (0x0000000007060500, 4),
        (0x0000000007060501, 4),
        (0x0000000706050100, 5),
        (0x0000000007060502, 4),
        (0x0000000706050200, 5),
        (0x0000000706050201, 5),
        (0x0000070605020100, 6),
        (0x0000000007060503, 4),
        (0x0000000706050300, 5),
        (0x0000000706050301, 5),
        (0x0000070605030100, 6),
        (0x0000000706050302, 5),
        (0x0000070605030200, 6),
        (0x0000070605030201, 6),
        (0x0007060503020100, 7),
        (0x0000000007060504, 4),
        (0x0000000706050400, 5),
        (0x0000000706050401, 5),
        (0x0000070605040100, 6),
        (0x0000000706050402, 5),
        (0x0000070605040200, 6),
        (0x0000070605040201, 6),
        (0x0007060504020100, 7),
        (0x0000000706050403, 5),
        (0x0000070605040300, 6),
        (0x0000070605040301, 6),
        (0x0007060504030100, 7),
        (0x0000070605040302, 6),
        (0x0007060504030200, 7),
        (0x0007060504030201, 7),
        (0x0706050403020100, 8),
    ];

    const ALL_BYTES_SET_TO_1: u64 = 0x0101010101010101;
    let offset_adj = ALL_BYTES_SET_TO_1 * (index_offset as u64);

    let (indices, count) = INDEX_LOOKUP_MAP.get_unchecked((partiton_mask) as usize);
    let indices_adjusted = indices + offset_adj;

    (offsets_ptr as *mut u64).write(indices_adjusted);
    offsets_ptr = offsets_ptr.add(*count as usize);

    offsets_ptr
}

#[inline]
#[target_feature(enable = "avx2")]
#[cfg(target_arch = "x86_64")]
unsafe fn analyze_block<T, F>(
    v_block: &[T],
    pivot: &T,
    mut offsets_ptr: *mut u8,
    is_less: &mut F,
) -> *mut u8
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: offsets_ptr must be able to hold block.len() writes. and bmi2 must be supported.
    use core::arch::x86_64;

    let block_len = v_block.len();
    assert!(block_len <= u8::MAX as usize);

    const UNROLL_SIZE: usize = 32;

    let mut unroll_offset = 0;

    if block_len >= UNROLL_SIZE {
        let unroll_end = block_len - UNROLL_SIZE;

        let mut comp_results = mem::MaybeUninit::<[u8; UNROLL_SIZE]>::uninit();
        let comp_results_ptr = comp_results.as_mut_ptr() as *mut u8;

        while unroll_offset < unroll_end {
            // SAFETY: TODO
            unsafe {
                for i in 0..UNROLL_SIZE {
                    comp_results_ptr.add(i).write(
                        is_less(v_block.get_unchecked(unroll_offset + i), pivot) as u8 * u8::MAX,
                    );
                }

                // Each byte is either 0u8 -> is_partitioned or all bits set 255u8 -> not is_partitioned.
                let is_offset_simd =
                    x86_64::_mm256_lddqu_si256(comp_results_ptr as *const x86_64::__m256i);
                let is_offset_packed: i32 = x86_64::_mm256_movemask_epi8(is_offset_simd);

                let lookup_bytes = mem::transmute::<i32, [u8; 4]>(is_offset_packed);
                for i in 0..4 {
                    offsets_ptr =
                        update_offsets_ptr(lookup_bytes[i], offsets_ptr, unroll_offset + (i * 8));
                }
            }

            unroll_offset += UNROLL_SIZE;
        }
    }

    for i in unroll_offset..block_len {
        // SAFETY: The unsafety operations below involve the usage of the `offset`.
        //         According to the conditions required by the function, we satisfy them because:
        //         1. `offsets_l` is stack-allocated, and thus considered separate allocated object.
        //         2. The function `is_less` returns a `bool`.
        //            Casting a `bool` will never overflow `isize`.
        //         3. We have guaranteed that `block_l` will be `<= BLOCK`.
        //            Plus, `end_l` was initially set to the begin pointer of `offsets_` which was declared on the stack.
        //            Thus, we know that even in the worst case (all invocations of `is_less` returns false) we will only be at most 1 byte pass the end.
        //        Another unsafety operation here is dereferencing `elem`.
        //        However, `elem` was initially the begin pointer to the slice which is always valid.
        unsafe {
            // Branchless comparison.
            *offsets_ptr = i as u8;
            offsets_ptr =
                offsets_ptr.wrapping_add(is_less(v_block.get_unchecked(i), pivot) as usize);
        }
    }

    offsets_ptr
}

#[target_feature(enable = "avx2")]
#[cfg(target_arch = "x86_64")]
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
unsafe fn partition_avx2<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    if !is_x86_feature_detected!("avx2") {
        panic!("Unsupported platform");
    }

    // Number of elements in a typical block.
    const BLOCK: usize = 256 - 32;

    // The partitioning algorithm repeats the following steps until completion:
    //
    // 1. Trace a block from the left side to identify elements greater than or equal to the pivot.
    // 2. Trace a block from the right side to identify elements smaller than the pivot.
    // 3. Exchange the identified elements between the left and right side.
    //
    // We keep the following variables for a block of elements:
    //
    // 1. `block` - Number of elements in the block.
    // 2. `start` - Start pointer into the `offsets` array.
    // 3. `end` - End pointer into the `offsets` array.
    // 4. `offsets - Indices of out-of-order elements within the block.

    // The current block on the left side (from `l` to `l.add(block_l)`).
    let mut l = v.as_mut_ptr();
    let mut block_l = BLOCK;
    let mut start_l = ptr::null_mut();
    let mut end_l = ptr::null_mut();
    let mut offsets_l = [MaybeUninit::<u8>::uninit(); BLOCK];

    // The current block on the right side (from `r.sub(block_r)` to `r`).
    // SAFETY: The documentation for .add() specifically mention that `vec.as_ptr().add(vec.len())` is always safe`
    let mut r = unsafe { l.add(v.len()) };
    let mut block_r = BLOCK;
    let mut start_r = ptr::null_mut();
    let mut start_r_rev = ptr::null_mut();
    let mut r_block_start = ptr::null_mut();
    let mut end_r = ptr::null_mut();
    let mut offsets_r = [MaybeUninit::<u8>::uninit(); BLOCK];

    // FIXME: When we get VLAs, try creating one array of length `min(v.len(), 2 * BLOCK)` rather
    // than two fixed-size arrays of length `BLOCK`. VLAs might be more cache-efficient.

    // Returns the number of elements between pointers `l` (inclusive) and `r` (exclusive).
    fn width<T>(l: *mut T, r: *mut T) -> usize {
        debug_assert!(r.addr() >= l.addr());

        unsafe { r.sub_ptr(l) }
    }

    loop {
        // We are done with partitioning block-by-block when `l` and `r` get very close. Then we do
        // some patch-up work in order to partition the remaining elements in between.
        let is_done = width(l, r) <= 2 * BLOCK;

        if is_done {
            // Number of remaining elements (still not compared to the pivot).
            let mut rem = width(l, r);
            if start_l < end_l || start_r < end_r {
                rem -= BLOCK;
            }

            // Adjust block sizes so that the left and right block don't overlap, but get perfectly
            // aligned to cover the whole remaining gap.
            if start_l < end_l {
                block_r = rem;
            } else if start_r < end_r {
                block_l = rem;
            } else {
                // There were the same number of elements to switch on both blocks during the last
                // iteration, so there are no remaining elements on either block. Cover the remaining
                // items with roughly equally-sized blocks.
                block_l = rem / 2;
                block_r = rem - block_l;
            }
            debug_assert!(block_l <= BLOCK && block_r <= BLOCK);
            debug_assert!(width(l, r) == block_l + block_r);
        }

        if start_l == end_l {
            // Trace `block_l` elements from the left side.
            start_l = MaybeUninit::slice_as_mut_ptr(&mut offsets_l);
            end_l = unsafe {
                analyze_block(
                    &*ptr::slice_from_raw_parts(l, block_l),
                    pivot,
                    start_l,
                    &mut |a, b| !is_less(a, b),
                )
            };
        }

        if start_r == end_r {
            // Trace `block_r` elements from the right side.
            start_r = MaybeUninit::slice_as_mut_ptr(&mut offsets_r);
            end_r = start_r;

            end_r = unsafe {
                analyze_block(
                    &*ptr::slice_from_raw_parts(r.sub(block_r), block_r),
                    pivot,
                    start_r,
                    is_less,
                )
            };
            start_r_rev = end_r.sub(1);
            r_block_start = r.sub(block_r);
        }

        // Number of out-of-order elements to swap between the left and right side.
        let count = cmp::min(width(start_l, end_l), width(start_r, end_r));

        if count > 0 {
            macro_rules! left {
                () => {
                    l.add(*start_l as usize)
                };
            }
            macro_rules! right {
                () => {
                    r_block_start.add(*start_r_rev as usize)
                };
            }

            // Instead of swapping one pair at the time, it is more efficient to perform a cyclic
            // permutation. This is not strictly equivalent to swapping, but produces a similar
            // result using fewer memory operations.

            // SAFETY: The use of `ptr::read` is valid because there is at least one element in
            // both `offsets_l` and `offsets_r`, so `left!` is a valid pointer to read from.
            //
            // The uses of `left!` involve calls to `offset` on `l`, which points to the
            // beginning of `v`. All the offsets pointed-to by `start_l` are at most `block_l`, so
            // these `offset` calls are safe as all reads are within the block. The same argument
            // applies for the uses of `right!`.
            //
            // The calls to `start_l.offset` are valid because there are at most `count-1` of them,
            // plus the final one at the end of the unsafe block, where `count` is the minimum number
            // of collected offsets in `offsets_l` and `offsets_r`, so there is no risk of there not
            // being enough elements. The same reasoning applies to the calls to `start_r.offset`.
            //
            // The calls to `copy_nonoverlapping` are safe because `left!` and `right!` are guaranteed
            // not to overlap, and are valid because of the reasoning above.
            unsafe {
                let tmp = ptr::read(left!());
                ptr::copy_nonoverlapping(right!(), left!(), 1);

                for _ in 1..count {
                    start_l = start_l.offset(1);
                    ptr::copy_nonoverlapping(left!(), right!(), 1);
                    start_r_rev = start_r_rev.sub(1);
                    ptr::copy_nonoverlapping(right!(), left!(), 1);
                }

                ptr::copy_nonoverlapping(&tmp, right!(), 1);
                mem::forget(tmp);
                start_r_rev = start_r_rev.sub(1);

                start_l = start_l.add(1);
                start_r = start_r.add(count);
            }
        }

        if start_l == end_l {
            // All out-of-order elements in the left block were moved. Move to the next block.

            // block-width-guarantee
            // SAFETY: if `!is_done` then the slice width is guaranteed to be at least `2*BLOCK` wide. There
            // are at most `BLOCK` elements in `offsets_l` because of its size, so the `offset` operation is
            // safe. Otherwise, the debug assertions in the `is_done` case guarantee that
            // `width(l, r) == block_l + block_r`, namely, that the block sizes have been adjusted to account
            // for the smaller number of remaining elements.
            l = unsafe { l.add(block_l) };
        }

        if start_r == end_r {
            // All out-of-order elements in the right block were moved. Move to the previous block.

            // SAFETY: Same argument as [block-width-guarantee]. Either this is a full block `2*BLOCK`-wide,
            // or `block_r` has been adjusted for the last handful of elements.
            r = unsafe { r.sub(block_r) };
        }

        if is_done {
            break;
        }
    }

    // All that remains now is at most one block (either the left or the right) with out-of-order
    // elements that need to be moved. Such remaining elements can be simply shifted to the end
    // within their block.

    if start_l < end_l {
        // The left block remains.
        // Move its remaining out-of-order elements to the far right.
        debug_assert_eq!(width(l, r), block_l);
        while start_l < end_l {
            // remaining-elements-safety
            // SAFETY: while the loop condition holds there are still elements in `offsets_l`, so it
            // is safe to point `end_l` to the previous element.
            //
            // The `ptr::swap` is safe if both its arguments are valid for reads and writes:
            //  - Per the debug assert above, the distance between `l` and `r` is `block_l`
            //    elements, so there can be at most `block_l` remaining offsets between `start_l`
            //    and `end_l`. This means `r` will be moved at most `block_l` steps back, which
            //    makes the `r.offset` calls valid (at that point `l == r`).
            //  - `offsets_l` contains valid offsets into `v` collected during the partitioning of
            //    the last block, so the `l.offset` calls are valid.
            unsafe {
                end_l = end_l.sub(1);
                ptr::swap(l.add(*end_l as usize), r.sub(1));
                r = r.sub(1);
            }
        }
        width(v.as_mut_ptr(), r)
    } else if start_r < end_r {
        // The right block remains.
        // Move its remaining out-of-order elements to the far left.
        debug_assert_eq!(width(l, r), block_r);

        // SAFETY: Same argument as [block-width-guarantee]. Either this is a full block `2*BLOCK`-wide,
        // or `block_r` has been adjusted for the last handful of elements.
        // let r_block_start = unsafe { r.sub(block_r) };
        let end_r_rev = mem::MaybeUninit::slice_as_mut_ptr(&mut offsets_r);

        while start_r_rev >= end_r_rev {
            // SAFETY: See the reasoning in [remaining-elements-safety].
            unsafe {
                ptr::swap(l, r_block_start.add(*start_r_rev as usize));
                start_r_rev = start_r_rev.sub(1);
                l = l.add(1);
            }
        }
        width(v.as_mut_ptr(), l)
    } else {
        // Nothing else to do, we're done.
        width(v.as_mut_ptr(), l)
    }
}

#[cfg(not(target_arch = "x86_64"))]
unsafe fn partition_avx2<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    panic!("Unsupported platform");
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    unsafe { partition_avx2(v, pivot, is_less) }
}
