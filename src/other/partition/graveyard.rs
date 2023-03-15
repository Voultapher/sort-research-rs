use core::cmp;
use core::intrinsics;
use core::mem;
use core::ptr;

partition_impl!("new_block_quicksort");

// Uniform collect_offsets func.

/// Check elements of v and return array of offsets that return true for check(elem)
/// offsets_base_ptr must hold space for v.len u8.
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
unsafe fn collect_offsets<T, F>(v: &[T], offsets_base_ptr: *mut u8, check: &mut F) -> *mut u8
where
    F: FnMut(&T) -> bool,
{
    let len = v.len();
    debug_assert!(len <= u8::MAX as usize);

    let mut offsets_ptr = offsets_base_ptr;

    for i in 0..len {
        // SAFETY: v.len() must be at most u8::MAX. And the caller must ensure that
        // offsets_ptr[0..len] is valid to write.
        unsafe {
            offsets_ptr.write(i as u8);
            offsets_ptr = offsets_ptr.add(check(v.get_unchecked(i)) as usize);
        }
    }

    offsets_ptr
}

/// Partitions `v` into elements smaller than `pivot`, followed by elements greater than or equal
/// to `pivot`.
///
/// Returns the number of elements smaller than `pivot`.
///
/// Partitioning is performed block-by-block in order to minimize the cost of branching operations.
/// This idea is presented in the [BlockQuicksort][pdf] paper.
///
/// [pdf]: https://drops.dagstuhl.de/opus/volltexte/2016/6389/pdf/LIPIcs-ESA-2016-38.pdf
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // unsafe {
    //     let x = mem::transmute::<&[T], &[i32]>(v);
    //     let pivot_i32 = mem::transmute::<&T, &i32>(pivot);
    //     println!("{:?} {} {}", x, x.len(), pivot_i32);
    // }

    // Number of elements in a typical block.
    const BLOCK: usize = 128;

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
    let mut offsets_l = [mem::MaybeUninit::<u8>::uninit(); BLOCK];

    // The current block on the right side (from `r.sub(block_r)` to `r`).
    // SAFETY: The documentation for .add() specifically mention that `vec.as_ptr().add(vec.len())` is always safe`
    let mut r = unsafe { l.add(v.len()) };
    let mut block_r = BLOCK;
    let mut start_r = ptr::null_mut();
    let mut start_r_rev = ptr::null_mut();
    let mut end_r = ptr::null_mut();
    let mut offsets_r = [mem::MaybeUninit::<u8>::uninit(); BLOCK];

    // FIXME: When we get VLAs, try creating one array of length `min(v.len(), 2 * BLOCK)` rather
    // than two fixed-size arrays of length `BLOCK`. VLAs might be more cache-efficient.

    // Returns the number of elements between pointers `l` (inclusive) and `r` (exclusive).
    fn width<T>(l: *mut T, r: *mut T) -> usize {
        debug_assert!(r.addr() >= l.addr());

        // SAFETY: r >= l and not T::IS_ZST
        unsafe { intrinsics::ptr_offset_from_unsigned(r, l) }
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

            // SAFETY: offsets_l can be written for BLOCK elements. And the area of v is valid
            // because TODO.
            unsafe {
                start_l = mem::MaybeUninit::slice_as_mut_ptr(&mut offsets_l);
                end_l = collect_offsets(
                    &*ptr::slice_from_raw_parts(l, block_l),
                    start_l,
                    &mut |elem| !is_less(elem, pivot),
                );
            }
        }

        // SAFETY: Same argument as [block-width-guarantee]. Either this is a full block `2*BLOCK`-wide,
        // or `block_r` has been adjusted for the last handful of elements.
        let r_block_start = unsafe { r.sub(block_r) };

        if start_r == end_r {
            // Trace `block_r` elements from the right side.

            // SAFETY: offsets_r can be written for BLOCK elements. And the area of v is valid
            // because TODO.
            unsafe {
                start_r = mem::MaybeUninit::slice_as_mut_ptr(&mut offsets_r);
                end_r = collect_offsets(
                    &*ptr::slice_from_raw_parts(r_block_start, block_r),
                    start_r,
                    &mut |elem| is_less(elem, pivot),
                );
                start_r_rev = end_r.sub(1);
            }
        }

        // Number of out-of-order elements to swap between the left and right side.
        let count = cmp::min(width(start_l, end_l), width(start_r, end_r));

        if count > 0 {
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
                // Reverse iterate through right window offsets to simplify collect_offsets.
                //
                // Because count is > 0 we know at least one out-of-order element exists on both sides.
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

                let tmp = ptr::read(left!());
                ptr::copy_nonoverlapping(right!(), left!(), 1);

                // dbg!(block_l, block_r, count);
                for _ in 1..count {
                    start_l = start_l.add(1);
                    // println!(
                    //     "left {} -> right {}",
                    //     *start_l,
                    //     width(v.as_mut_ptr(), right!())
                    // );
                    ptr::copy_nonoverlapping(left!(), right!(), 1);
                    start_r_rev = start_r_rev.sub(1);
                    // println!(
                    //     "right {} -> left {}",
                    //     width(v.as_mut_ptr(), right!()),
                    //     *start_l
                    // );
                    ptr::copy_nonoverlapping(right!(), left!(), 1);
                }

                ptr::copy_nonoverlapping(&tmp, right!(), 1);
                mem::forget(tmp);
                start_r_rev = start_r_rev.sub(1);

                start_l = start_l.add(1);
                start_r = start_r.add(count);

                // let l_offset = width(offsets_l.as_mut_ptr() as *mut u8, start_l);
                // let r_offset = width(offsets_r.as_mut_ptr() as *mut u8, start_r);
                // dbg!(l_offset, r_offset);
            }
        }

        // unsafe {
        //     let x = mem::transmute::<&[T], &[i32]>(v);
        //     let pivot_i32 = mem::transmute::<&T, &i32>(pivot);
        //     println!("{:?} {} {}", x, x.len(), pivot_i32);
        // }

        // TODO check perf of full double block refill.
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
            r = r_block_start;
        }

        if is_done {
            break;
        }
    }

    // All that remains now is at most one block (either the left or the right) with out-of-order
    // elements that need to be moved. Such remaining elements can be simply shifted to the end
    // within their block.
    // dbg!(width(v.as_mut_ptr(), l), width(v.as_mut_ptr(), r));

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
        let r_block_start = unsafe { r.sub(block_r) };
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

fn xx() {
    // Nice idea but really slow.
    let mut swap = mem::MaybeUninit::<[T; SWAP]>::uninit();
    let mut swap_ptr = swap.as_mut_ptr() as *mut T;

    let mut offsets_r = mem::MaybeUninit::<[u8; SWAP]>::uninit();
    let mut offsets_ptr = offsets_r.as_mut_ptr() as *mut u8;

    for i in 0..len {
        unsafe {
            let value = v.get_unchecked(i);

            let is_l = is_less(value, pivot);

            swap_ptr.write(*value);
            offsets_ptr.write(i as u8);

            swap_ptr = swap_ptr.add(is_l as usize);
            offsets_ptr = offsets_ptr.add(!is_l as usize);
        }
    }

    // SAFETY: swap now contains all elements that belong on the left side of the pivot. All
    // comparisons have been done if is_less would have panicked v would have stayed untouched.
    unsafe {
        let arr_ptr = v.as_mut_ptr();
        let l_elems = swap_ptr.sub_ptr(swap.as_ptr() as *const T);
        let r_elems = offsets_ptr.sub_ptr(offsets_r.as_ptr() as *const u8);

        let offsets_base_ptr = offsets_r.as_ptr() as *const u8;

        for i in 0..r_elems {
            ptr::copy_nonoverlapping(
                arr_ptr.add(*offsets_base_ptr.add(i) as usize),
                swap_ptr.add(i),
                1,
            );
        }

        // Now that swap has the correct order overwrite arr_ptr.
        ptr::copy_nonoverlapping(swap.as_ptr() as *const T, arr_ptr, len);

        l_elems
    }
}

// --- lookup table analyze ---
use core::cmp;
// use core::intrinsics;
use core::mem::{self, MaybeUninit};
use core::ptr;

partition_impl!("new_block_quicksort");

// #[inline]
// unsafe fn update_offsets_ptr_impl(
//     partiton_mask: u8,
//     mut offsets_ptr: *mut u8,
//     mut offset_adj: u64,
// ) -> (*mut u8, u64) {
//     let (indices, count) = INDEX_LOOKUP_MAP.get_unchecked((partiton_mask) as usize);
//     let indices_adjusted = indices + offset_adj;

//     (offsets_ptr as *mut u64).write(indices_adjusted);
//     offsets_ptr = offsets_ptr.add(*count as usize);
//     offset_adj += 289360691352306692; // [4, 4, 4, 4, 4, 4, 4, 4] as u64

//     (offsets_ptr, offset_adj)
// }

#[inline]
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

#[target_feature(enable = "bmi2")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn analyze_block<T, F>(
    block: &[T],
    pivot: &T,
    mut offsets_ptr: *mut u8,
    is_less: &mut F,
) -> *mut u8
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: offsets_ptr must be able to hold block.len() writes. and bmi2 must be supported.
    use core::arch::x86_64;

    let block_len = block.len();
    assert!(block_len <= u8::MAX as usize);

    let mut elem_ptr = block.as_ptr();

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
                    comp_results_ptr
                        .add(i)
                        .write(!is_less(&*elem_ptr.add(unroll_offset + i), pivot) as u8 * u8::MAX);
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

    // SAFETY: TODO
    unsafe {
        elem_ptr = elem_ptr.add(unroll_offset);
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
            offsets_ptr = offsets_ptr.wrapping_add(!is_less(&*elem_ptr, pivot) as usize);
            elem_ptr = elem_ptr.add(1);
        }
    }

    offsets_ptr
}

/// Partitions `v` into elements smaller than `pivot`, followed by elements greater than or equal
/// to `pivot`.
///
/// Returns the number of elements smaller than `pivot`.
///
/// Partitioning is performed block-by-block in order to minimize the cost of branching operations.
/// This idea is presented in the [BlockQuicksort][pdf] paper.
///
/// [pdf]: https://drops.dagstuhl.de/opus/volltexte/2016/6389/pdf/LIPIcs-ESA-2016-38.pdf
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
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
                    is_less,
                )
            };
        }

        if start_r == end_r {
            // Trace `block_r` elements from the right side.
            start_r = MaybeUninit::slice_as_mut_ptr(&mut offsets_r);
            end_r = start_r;
            let mut elem = r;

            for i in 0..block_r {
                // SAFETY: The unsafety operations below involve the usage of the `offset`.
                //         According to the conditions required by the function, we satisfy them because:
                //         1. `offsets_r` is stack-allocated, and thus considered separate allocated object.
                //         2. The function `is_less` returns a `bool`.
                //            Casting a `bool` will never overflow `isize`.
                //         3. We have guaranteed that `block_r` will be `<= BLOCK`.
                //            Plus, `end_r` was initially set to the begin pointer of `offsets_` which was declared on the stack.
                //            Thus, we know that even in the worst case (all invocations of `is_less` returns true) we will only be at most 1 byte pass the end.
                //        Another unsafety operation here is dereferencing `elem`.
                //        However, `elem` was initially `1 * sizeof(T)` past the end and we decrement it by `1 * sizeof(T)` before accessing it.
                //        Plus, `block_r` was asserted to be less than `BLOCK` and `elem` will therefore at most be pointing to the beginning of the slice.
                unsafe {
                    // Branchless comparison.
                    elem = elem.sub(1);
                    *end_r = i as u8;
                    end_r = end_r.wrapping_add(is_less(&*elem, pivot) as usize);
                }
            }
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
                    r.sub(*start_r as usize + 1)
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
                    start_l = start_l.add(1);
                    ptr::copy_nonoverlapping(left!(), right!(), 1);
                    start_r = start_r.add(1);
                    ptr::copy_nonoverlapping(right!(), left!(), 1);
                }

                ptr::copy_nonoverlapping(&tmp, right!(), 1);
                mem::forget(tmp);
                start_l = start_l.add(1);
                start_r = start_r.add(1);
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
        while start_r < end_r {
            // SAFETY: See the reasoning in [remaining-elements-safety].
            unsafe {
                end_r = end_r.sub(1);
                ptr::swap(l, r.sub(*end_r as usize + 1));
                l = l.add(1);
            }
        }
        width(v.as_mut_ptr(), l)
    } else {
        // Nothing else to do, we're done.
        width(v.as_mut_ptr(), l)
    }
}

// Wow this is suprisingly slow, I guess it doesn't like the changing write location in fill_block.

use core::cmp;
use core::mem;
use core::ptr;

partition_impl!("new_block_quicksort");

const U8_BITS_USED: usize = 8;
const U8_COMBINATIONS: usize = 2usize.pow(u8::BITS);
const BLOCK: usize = 16;
const BLOCK_ELEMS: usize = BLOCK * U8_BITS_USED;

#[derive(Copy, Clone)]
struct BlockEntry {
    partition_mask: u8,
    offset: u8,
}

#[inline]
unsafe fn gen_partition_mask<T, F>(block_ptr: *const T, is_out_of_order: &mut F) -> u8
where
    F: FnMut(&T) -> bool,
{
    let mut partition_mask = 0;

    // This should be unrolled by the optimizer.
    // TODO try out smaller spread for better occupancy.
    for i in 0..U8_BITS_USED {
        let elem: &T = unsafe { &*block_ptr.add(i) };
        partition_mask |= (is_out_of_order(elem) as u8).wrapping_shl(i as u32);
    }

    partition_mask
}

#[inline]
unsafe fn fill_block<T, F>(
    base_ptr: *const T,
    blocks_ptrs: *mut *mut BlockEntry,
    is_out_of_order: &mut F,
) where
    F: FnMut(&T) -> bool,
{
    for i in 0..BLOCK as u8 {
        // TODO check if it is cheaper to mut base_ptr instead of mult here.
        unsafe {
            let partition_mask =
                gen_partition_mask(base_ptr.add(i as usize * U8_BITS_USED), is_out_of_order);

            // TODO check branchless version.
            // if partition_mask == 0 {
            //     continue;
            // }

            let block_entry = BlockEntry {
                partition_mask,
                offset: i,
            };

            let bucket = U8_BIT_COUNT_TABLE
                .get_unchecked(partition_mask as usize)
                .saturating_sub(1);

            let bucket_ptr = blocks_ptrs.add(bucket as usize);
            (*bucket_ptr).write(block_entry);
            bucket_ptr.write((*bucket_ptr).add(1));
        }
    }
}

fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // Returns the number of elements between pointers `l` (inclusive) and `r` (exclusive).
    fn width<T>(l: *const T, r: *const T) -> usize {
        debug_assert!(r.addr() >= l.addr());

        unsafe { r.sub_ptr(l) }
    }

    // TODO check smaller sizes right now this uses BLOCK * 16 * 8 * 2 stack space.
    let mut blocks_l = [mem::MaybeUninit::<[BlockEntry; BLOCK]>::uninit(); U8_BITS_USED];
    let mut blocks_l_ptrs = [mem::MaybeUninit::<*mut BlockEntry>::uninit(); U8_BITS_USED];

    let mut blocks_r = [mem::MaybeUninit::<[BlockEntry; BLOCK]>::uninit(); U8_BITS_USED];
    let mut blocks_r_ptrs = [mem::MaybeUninit::<*mut BlockEntry>::uninit(); U8_BITS_USED];

    let len = v.len();

    let mut l_base_ptr = v.as_ptr();
    let mut r_base_ptr = unsafe { v.as_ptr().add(len.saturating_sub(BLOCK_ELEMS)) };

    let mut side_effect = 0;

    while width(l_base_ptr, r_base_ptr) >= (BLOCK_ELEMS * 2) {
        // Reset blocks pointers
        for i in 0..U8_BITS_USED {
            unsafe {
                *blocks_l_ptrs[i].as_mut_ptr() = blocks_l[i].as_mut_ptr() as *mut BlockEntry;
                *blocks_r_ptrs[i].as_mut_ptr() = blocks_r[i].as_mut_ptr() as *mut BlockEntry;
            }
        }

        unsafe {
            fill_block(
                l_base_ptr,
                blocks_l_ptrs.as_mut_ptr() as *mut *mut BlockEntry,
                &mut |elem| !is_less(elem, pivot),
            );

            fill_block(
                r_base_ptr,
                blocks_r_ptrs.as_mut_ptr() as *mut *mut BlockEntry,
                &mut |elem| is_less(elem, pivot),
            );
        }

        // Now blocks_l and blocks_r should contain BlockEntry in their associated bucket
        // denoting how many elements are out-of-order. Match up bucket entries that have
        // the same amount of out-of-order entries. There is no bucket for zero elements are
        // out-of-order. Those should just stay in-place.
        let calc_block_count_l = |bucket: usize| {
            let l_block_base_ptr = blocks_l[bucket].as_ptr() as *mut BlockEntry;
            unsafe { width(l_block_base_ptr, blocks_l_ptrs[bucket].assume_init()) }
        };

        let calc_block_count_r = |bucket: usize| {
            let r_block_base_ptr = blocks_r[bucket].as_ptr() as *mut BlockEntry;
            unsafe { width(r_block_base_ptr, blocks_r_ptrs[bucket].assume_init()) }
        };

        let calc_block_count = |bucket| {
            let count_l = calc_block_count_l(bucket);
            let count_r = calc_block_count_r(bucket);
            (count_l, count_r, cmp::min(count_l, count_r))
        };

        // Debug
        // for i in 0..U8_BITS_USED {
        //     let l_block_count = calc_block_count_l(i);
        //     let r_block_count = calc_block_count_r(i);

        //     println!("[{i}] l_block_count: {l_block_count} r_block_count: {r_block_count}");
        // }

        let l_block_base_ptr_8 = blocks_l[7].as_ptr() as *mut BlockEntry;
        let r_block_base_ptr_8 = blocks_r[7].as_ptr() as *mut BlockEntry;
        let (block_count_8_l, block_count_8_r, block_count_8_min) = calc_block_count(7);
        // for i in 0..block_count_8_min {
        //     unsafe {
        //         ptr::swap_nonoverlapping(l_block_base_ptr_8.add(i), r_block_base_ptr_8.add(8), 8);
        //     }
        // }

        side_effect += block_count_8_min;

        unsafe {
            l_base_ptr = l_base_ptr.add(BLOCK_ELEMS);
            r_base_ptr = r_base_ptr.sub(BLOCK_ELEMS);
        }
    }

    // FIXME for test
    // <crate::other::partition::block_quicksort::PartitionImpl as crate::other::partition::Partition>::partition_by(v, pivot, is_less)
    side_effect
}

// Using a lookup table is significantly faster than .count_ones()
const U8_BIT_COUNT_TABLE: [u8; U8_COMBINATIONS] = [
    0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, 1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7, 4, 5, 5, 6, 5, 6, 6, 7, 5, 6, 6, 7, 6, 7, 7, 8,
];

// Fast swapping.
/// TODO explain
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
#[inline(always)]
unsafe fn swap_elements_between_blocks<T>(
    l_ptr: *mut T,
    r_ptr: *mut T,
    mut l_offsets_ptr: *const u8,
    mut r_offsets_ptr: *const u8,
    count: usize,
) -> (*const u8, *const u8) {
    macro_rules! left {
        ($offset_ptr:expr) => {
            l_ptr.add(*$offset_ptr as usize)
        };
    }
    macro_rules! right {
        ($offset_ptr:expr) => {
            r_ptr.sub(*$offset_ptr as usize)
        };
    }

    // if count == 0 {
    //     // if count == 1 {
    //     //     // SAFETY: TODO
    //     //     unsafe {
    //     //         ptr::swap_nonoverlapping(left!(), right!(), 1);
    //     //         l_offsets_ptr = l_offsets_ptr.add(1);
    //     //         r_offsets_ptr = r_offsets_ptr.add(1);
    //     //     }
    //     // }

    //     return (l_offsets_ptr, r_offsets_ptr);
    // }

    let even_count = count - (count % 2 != 0) as usize;

    unsafe {
        if even_count >= 2 {
            // Save the first two elements from the left for later.
            let tmp_a = ptr::read(left!(l_offsets_ptr.add(0)));
            let tmp_b = ptr::read(left!(l_offsets_ptr.add(1)));

            // Copy two elements from right onto of saved elements.
            ptr::copy_nonoverlapping(right!(r_offsets_ptr.add(0)), left!(l_offsets_ptr.add(0)), 1);
            ptr::copy_nonoverlapping(right!(r_offsets_ptr.add(1)), left!(l_offsets_ptr.add(1)), 1);

            l_offsets_ptr = l_offsets_ptr.add(2);

            let mut i = 2;
            while i < even_count {
                // Copy two elements from left to right.
                ptr::copy_nonoverlapping(left!(l_offsets_ptr), right!(r_offsets_ptr), 1);
                ptr::copy_nonoverlapping(
                    left!(l_offsets_ptr.add(1)),
                    right!(r_offsets_ptr.add(1)),
                    1,
                );

                // Copy two elements from right to left.
                ptr::copy_nonoverlapping(right!(r_offsets_ptr.add(2)), left!(l_offsets_ptr), 1);
                ptr::copy_nonoverlapping(
                    right!(r_offsets_ptr.add(3)),
                    left!(l_offsets_ptr.add(1)),
                    1,
                );

                i += 2;
                l_offsets_ptr = l_offsets_ptr.add(2);
                r_offsets_ptr = r_offsets_ptr.add(2);
            }

            // Copy saved elements to right side.
            ptr::copy_nonoverlapping(&tmp_a, right!(r_offsets_ptr), 1);
            ptr::copy_nonoverlapping(&tmp_b, right!(r_offsets_ptr.add(1)), 1);

            mem::forget(tmp_a);
            mem::forget(tmp_b);
            // l_offsets_ptr = l_offsets_ptr.add(2);
            r_offsets_ptr = r_offsets_ptr.add(2);
        }

        if even_count != count {
            unsafe {
                ptr::swap_nonoverlapping(left!(l_offsets_ptr), right!(r_offsets_ptr), 1);
            }
            l_offsets_ptr = l_offsets_ptr.add(1);
            r_offsets_ptr = r_offsets_ptr.add(1);
        }

        // (l_offsets_ptr.add(count), r_offsets_ptr.add(count))
        (l_offsets_ptr, r_offsets_ptr)
    }

    // // Instead of swapping one pair at the time, it is more efficient to perform a cyclic
    // // permutation. This is not strictly equivalent to swapping, but produces a similar
    // // result using fewer memory operations.

    // // SAFETY: The use of `ptr::read` is valid because there is at least one element in
    // // both `offsets_l` and `offsets_r`, so `left!` is a valid pointer to read from.
    // //
    // // The uses of `left!` involve calls to `offset` on `l`, which points to the
    // // beginning of `v`. All the offsets pointed-to by `l_offsets_ptr` are at most `block_l`, so
    // // these `offset` calls are safe as all reads are within the block. The same argument
    // // applies for the uses of `right!`.
    // //
    // // The calls to `l_offsets_ptr.offset` are valid because there are at most `count-1` of them,
    // // plus the final one at the end of the unsafe block, where `count` is the minimum number
    // // of collected offsets in `offsets_l` and `offsets_r`, so there is no risk of there not
    // // being enough elements. The same reasoning applies to the calls to `r_offsets_ptr.offset`.
    // //
    // // The calls to `copy_nonoverlapping` are safe because `left!` and `right!` are guaranteed
    // // not to overlap, and are valid because of the reasoning above.
    // unsafe {
    //     let tmp = ptr::read(left!());
    //     ptr::copy_nonoverlapping(right!(), left!(), 1);

    //     // println!("");
    //     for _ in 1..count {
    //         l_offsets_ptr = l_offsets_ptr.add(1);
    //         let a = *l_offsets_ptr;
    //         let b = *r_offsets_ptr;
    //         ptr::copy_nonoverlapping(left!(), right!(), 1);
    //         r_offsets_ptr = r_offsets_ptr.add(1);

    //         let x = *l_offsets_ptr;
    //         let y = *r_offsets_ptr;
    //         ptr::copy_nonoverlapping(right!(), left!(), 1);

    //         // println!("copied l {a} -> r {b} and r {y} -> l {x}");
    //     }

    //     ptr::copy_nonoverlapping(&tmp, right!(), 1);
    //     mem::forget(tmp);
    //     l_offsets_ptr = l_offsets_ptr.add(1);
    //     r_offsets_ptr = r_offsets_ptr.add(1);
    // }

    // if count > 0 {
    //     for i in 0..count {
    //         let r_elem_ptr = ptr::swap_nonoverlapping(
    //             l_ptr.add(*l_offsets_ptr.add(i) as usize),
    //             r_ptr.sub(*r_offsets_ptr.add(i) as usize + 1),
    //             1,
    //         );
    //     }
    //     l_offsets_ptr = l_offsets_ptr.add(count);
    //     r_offsets_ptr = r_offsets_ptr.add(count);
    // }

    // (l_offsets_ptr, r_offsets_ptr)
}

unsafe fn small_aux_partition<T, F>(
    v: &mut [T],
    swap_ptr: *mut T,
    pivot: &T,
    is_less: &mut F,
) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: TODO
    unsafe {
        let len = v.len();
        let even_len = len - (len % 2 != 0) as usize;
        let len_div_2 = even_len / 2;

        let arr_ptr = v.as_mut_ptr();

        let mut swap_ptr_l_a = swap_ptr;
        let mut swap_ptr_r_a = swap_ptr.add(len_div_2 - 1);

        let mut swap_ptr_l_b = swap_ptr.add(len_div_2);
        let mut swap_ptr_r_b = swap_ptr.add(even_len - 1);

        // This could probably be sped-up by interleaving the two loops.
        for i in 0..len_div_2 {
            let elem_ptr_a = arr_ptr.add(i);
            let is_l_a = is_less(&*elem_ptr_a, pivot);
            let target_ptr_a = if is_l_a { swap_ptr_l_a } else { swap_ptr_r_a };
            ptr::copy_nonoverlapping(elem_ptr_a, target_ptr_a, 1);
            swap_ptr_l_a = swap_ptr_l_a.add(is_l_a as usize);
            swap_ptr_r_a = swap_ptr_r_a.sub(!is_l_a as usize);

            let elem_ptr_b = arr_ptr.add(i);
            let is_l_b = is_less(&*elem_ptr_b, pivot);
            let target_ptr_b = if is_l_b { swap_ptr_l_b } else { swap_ptr_r_b };
            ptr::copy_nonoverlapping(elem_ptr_b, target_ptr_b, 1);
            swap_ptr_l_b = swap_ptr_l_b.add(is_l_b as usize);
            swap_ptr_r_b = swap_ptr_r_b.sub(!is_l_b as usize);
        }

        // Swap now contains [l_values_a, r_values_a, l_values_b, r_values_b]
        let is_l_count_a = swap_ptr_l_a.sub_ptr(swap_ptr);
        let is_l_count_b = swap_ptr_l_b.sub_ptr(swap_ptr) - len_div_2;

        let mut is_l_count = is_l_count_a + is_l_count_b;

        // Copy swap into v in correct order.

        // l_values_a
        ptr::copy_nonoverlapping(swap_ptr, arr_ptr, is_l_count_a);

        // l_values_b
        ptr::copy_nonoverlapping(
            swap_ptr.add(len_div_2),
            arr_ptr.add(is_l_count_a),
            is_l_count_b,
        );

        // r_values_a
        ptr::copy_nonoverlapping(
            swap_ptr.add(is_l_count_a),
            arr_ptr.add(is_l_count),
            len_div_2 - is_l_count_a,
        );

        // r_values_b
        ptr::copy_nonoverlapping(
            swap_ptr.add(len_div_2 + is_l_count_b),
            arr_ptr.add(is_l_count + (len_div_2 - is_l_count_a)),
            len_div_2 - is_l_count_b,
        );

        if even_len != len {
            if is_less(&v[even_len], pivot) {
                v.swap(is_l_count, even_len);
                is_l_count += 1;
            }
        }

        is_l_count
    }
}
