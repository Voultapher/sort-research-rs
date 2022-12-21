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
