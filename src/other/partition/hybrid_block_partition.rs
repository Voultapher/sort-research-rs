//! Conceptually the same Hoare partition + cyclic permutation used in BlockQuickSort. TODO explain.

use std::cmp;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ptr;

partition_impl!("hybrid_block_partition");

#[inline(always)]
unsafe fn fill_offset_block_up<const BLOCK: usize, T>(
    base_ptr: *const T,
    mut offset_out_ptr: *mut u8,
    is_swap_elem: &mut impl FnMut(&T) -> bool,
) -> (*mut u8, *mut u8) {
    let offset_base_ptr = offset_out_ptr;

    const UNROLL_LEN: usize = 8;

    for block_i in 0..(BLOCK / UNROLL_LEN) {
        let unroll_offset = block_i * UNROLL_LEN;

        for unroll_i in 0..UNROLL_LEN {
            let up_i = unroll_offset + unroll_i;
            *offset_out_ptr = up_i as u8;
            let is_se = is_swap_elem(&*base_ptr.add(up_i));
            offset_out_ptr = offset_out_ptr.add(is_se as usize);
        }
    }

    // for i in 0..BLOCK {
    //         let up_i = i;
    //         *offset_out_ptr = up_i as u8;
    //         let is_se = is_swap_elem(&*base_ptr.add(up_i));
    //         offset_out_ptr = offset_out_ptr.add(is_se as usize);
    // }

    (offset_base_ptr, offset_out_ptr)
}

#[inline(always)]
unsafe fn fill_offset_block_down<const BLOCK: usize, T>(
    base_ptr: *const T,
    mut offset_out_ptr: *mut u8,
    is_swap_elem: &mut impl FnMut(&T) -> bool,
) -> (*mut u8, *mut u8) {
    let offset_base_ptr = offset_out_ptr;

    const UNROLL_LEN: usize = 8;

    for block_i in 0..(BLOCK / UNROLL_LEN) {
        let unroll_offset = (((BLOCK / UNROLL_LEN) - 1) - block_i) * UNROLL_LEN;

        for unroll_i in 0..UNROLL_LEN {
            let down_i = unroll_offset + ((UNROLL_LEN - 1) - unroll_i);
            *offset_out_ptr = down_i as u8;
            let is_se = is_swap_elem(&*base_ptr.add(down_i));
            offset_out_ptr = offset_out_ptr.add(is_se as usize);
        }
    }

    // for i in 0..BLOCK {
    //     let down_i = (BLOCK - 1) - i;
    //     *offset_out_ptr = down_i as u8;
    //     let is_se = is_swap_elem(&*base_ptr.add(down_i));
    //     offset_out_ptr = offset_out_ptr.add(is_se as usize);
    // }

    (offset_base_ptr, offset_out_ptr)
}

/// SAFETY: The caller must ensure that all provided expression are no-panic and may not modify the
/// values produced by `next_left` and `next_right`. And the produced pointers MUST NOT alias.
fn cyclic_permutation_swap_loop<T>(
    l_ptr: *mut T,
    l_offsets_ptr: *const u8,
    r_ptr: *mut T,
    r_offsets_ptr: *const u8,
    swap_count: usize,
) {
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

    const UNROLL_LEN: usize = 8;

    // SAFETY: See function description
    unsafe {
        macro_rules! left {
            ($i:expr) => {
                l_ptr.add(*l_offsets_ptr.add($i) as usize)
            };
        }

        macro_rules! right {
            ($i:expr) => {
                r_ptr.add(*r_offsets_ptr.add($i) as usize)
            };
        }

        let mut i = 0;
        if i < swap_count {
            let mut left_ptr = left!(i);
            let mut right_ptr = right!(i);

            // SAFETY: The following code is both panic- and observation-safe, so it's ok to
            // create a temporary.
            let tmp = ManuallyDrop::new(ptr::read(left_ptr));
            ptr::copy_nonoverlapping(right_ptr, left_ptr, 1);

            macro_rules! loop_body {
                ($i:expr) => {
                    left_ptr = left!($i);
                    ptr::copy_nonoverlapping(left_ptr, right_ptr, 1);
                    right_ptr = right!($i);
                    ptr::copy_nonoverlapping(right_ptr, left_ptr, 1);
                };
            }

            i += 1;
            while i < swap_count {
                for unroll_i in 0..UNROLL_LEN {
                    loop_body!(unroll_i + i);
                }

                i += UNROLL_LEN;
            }

            // Avoid unrolling for the loop cleanup.
            let one = std::hint::black_box(1);
            while i < swap_count {
                loop_body!(i);
                i += one;
            }

            ptr::copy_nonoverlapping(&*tmp, right_ptr, 1);
        }
    }
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F: FnMut(&T, &T) -> bool>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize {
    const MIN_BLOCK_PARTITION_LEN: usize = 4096;
    const BLOCK: usize = 256;

    let len = v.len();
    let arr_ptr = v.as_mut_ptr();

    if len == 0 {
        return 0;
    }

    // lt == less than, ge == greater or equal
    // wse wrong side element
    let mut l_offset_buffer = MaybeUninit::<[u8; BLOCK]>::uninit();
    let l_offset_base_ptr = l_offset_buffer.as_mut_ptr() as *mut u8;
    let mut l_offset_start_ptr = l_offset_base_ptr;
    let mut l_offset_end_ptr = l_offset_base_ptr;

    let mut r_offset_buffer = MaybeUninit::<[u8; BLOCK]>::uninit();
    let r_offset_base_ptr = r_offset_buffer.as_mut_ptr() as *mut u8;
    let mut r_offset_start_ptr = r_offset_base_ptr;
    let mut r_offset_end_ptr = r_offset_base_ptr;

    // SAFETY: TODO
    unsafe {
        let mut l_ptr = arr_ptr;
        let mut remaining_len = len;

        if len >= MIN_BLOCK_PARTITION_LEN {
            let mut r_ptr = arr_ptr.add(len - BLOCK);

            // It's crucial for reliable auto-vectorization that BLOCK always stays the same. Which
            // means we handle the rest of the input size separately later.

            // If the region we will look at during this loop iteration overlaps we are done.
            while l_ptr.add(BLOCK) <= r_ptr {
                // loop { While interleaving left and right side access would be possible,
                // experiments show that on Zen3 this has significantly worse performance, and the
                // CPU prefers working on one region of memory followed by another.
                if l_offset_start_ptr == l_offset_end_ptr {
                    (l_offset_start_ptr, l_offset_end_ptr) =
                        fill_offset_block_up::<BLOCK, T>(l_ptr, l_offset_base_ptr, &mut |elem| {
                            !is_less(elem, pivot)
                        });
                }

                if r_offset_start_ptr == r_offset_end_ptr {
                    (r_offset_start_ptr, r_offset_end_ptr) =
                        fill_offset_block_down::<BLOCK, T>(r_ptr, r_offset_base_ptr, &mut |elem| {
                            is_less(elem, pivot)
                        });
                }

                let swap_count = cmp::min(
                    l_offset_end_ptr.offset_from_unsigned(l_offset_start_ptr),
                    r_offset_end_ptr.offset_from_unsigned(r_offset_start_ptr),
                );

                // TODO try out version that is manually unrolled to two.
                cyclic_permutation_swap_loop(
                    l_ptr,
                    l_offset_start_ptr,
                    r_ptr,
                    r_offset_start_ptr,
                    swap_count,
                );

                l_offset_start_ptr = l_offset_start_ptr.add(swap_count);
                r_offset_start_ptr = r_offset_start_ptr.add(swap_count);

                l_ptr = l_ptr.add((l_offset_start_ptr == l_offset_end_ptr) as usize * BLOCK);
                r_ptr = r_ptr.sub((r_offset_start_ptr == r_offset_end_ptr) as usize * BLOCK);
            }

            // TODO use leftover block info.
            remaining_len = r_ptr.add(BLOCK).offset_from_unsigned(l_ptr);
        }

        let outer_lt_count = l_ptr.offset_from_unsigned(arr_ptr);

        let inner_lt_count = <crate::other::partition::lomuto_branchless_cyclic::PartitionImpl as crate::other::partition::Partition>::partition_by(&mut *ptr::slice_from_raw_parts_mut(arr_ptr, remaining_len), pivot, is_less);

        let lt_count = outer_lt_count + inner_lt_count;

        lt_count
    }
}
