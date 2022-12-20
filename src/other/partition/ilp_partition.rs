#![allow(unused)]

use core::cmp;
use core::intrinsics;
use core::mem;
use core::ptr;
// use core::simd;

use crate::unstable::rust_new::branchless_swap;

partition_impl!("ilp_partition");

const OFFSET_SENTINEL: u8 = u8::MAX;

// unsafe fn collect_offsets_16<T, F>(v: &[T], pivot: &T, offsets_ptr: *mut u8, is_less: &mut F)
// where
//     F: FnMut(&T, &T) -> bool,
// {
//     debug_assert!(v.len() == BLOCK_SIZE);

//     // SAFETY: offsets_ptr must be able to hold 16 elements.
//     const BLOCK_SIZE: usize = 16;

//     // This should be unfolded by the optimizer.
//     for i in 0..BLOCK_SIZE {
//         offsets_ptr
//             .add(i)
//             .write((is_less(v.get_unchecked(i), pivot) as u8) * u8::MAX);
//     }
// }

// #[target_feature(enable = "avx2")]
// unsafe fn collect_offsets_32<T, F>(v: &[T], pivot: &T, offsets_ptr: *mut u8, is_less: &mut F)
// where
//     F: FnMut(&T, &T) -> bool,
// {
//     debug_assert!(v.len() == BLOCK_SIZE);

//     // SAFETY: offsets_ptr must be able to hold 16 elements.
//     const BLOCK_SIZE: usize = 32;

//     // This should be unfolded by the optimizer.
//     for i in 0..BLOCK_SIZE {
//         offsets_ptr
//             .add(i)
//             .write(is_less(v.get_unchecked(i), pivot) as u8);
//     }
// }

// /// Check 128 elements of v and return array of offsets that return true for check(elem, pivot)
// #[target_feature(enable = "avx2")]
// #[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
// unsafe fn collect_offsets_128<T, F>(
//     v: &[T],
//     pivot: &T,
//     check: &mut F,
// ) -> (mem::MaybeUninit<[u8; 128]>, usize)
// where
//     F: FnMut(&T, &T) -> bool,
// {
//     // SAFETY: Caller must ensure that v.len() is at least BLOCK_SIZE.
//     debug_assert!(v.len() >= BLOCK_SIZE);

//     use core::arch::x86_64;

//     // let mut offsets = [OFFSET_SENTINEL; N];
//     // let offsets_ptr = offsets.as_mut_ptr();

//     let arr_ptr = v.as_ptr();

//     // for offset in 0..(N as u8) {
//     //     let is_r_elem = !is_less(&*arr_ptr.add(offset as usize), pivot);
//     //     offsets_ptr.write(offset);
//     //     offsets_ptr = offsets_ptr.add(is_r_elem as usize);
//     // }
//     // let sum = intrinsics::ptr_offset_from_unsigned(offsets_ptr, offsets.as_mut_ptr());

//     // let mask = x86_64::__m128i::from(simd::u8x16::from([
//     //     1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
//     // ]));

//     const BLOCK_SIZE: usize = 32;
//     const N: usize = 128;

//     let mut offsets = mem::MaybeUninit::<[u8; N]>::uninit();
//     let mut offsets_ptr = offsets.as_mut_ptr() as *mut u8;

//     let mut sum = 0;
//     let mut block = 0;
//     while block < N {
//         let mut is_offset = mem::MaybeUninit::<[u8; BLOCK_SIZE]>::uninit();
//         let is_offset_ptr = is_offset.as_mut_ptr() as *mut u8;

//         for i in 0..BLOCK_SIZE {
//             is_offset_ptr
//                 .add(i)
//                 .write(check(v.get_unchecked(block + i), pivot) as u8 * u8::MAX);
//         }

//         // // Each byte is either 0u8 -> is_partitioned or all bits set 255u8 -> not is_partitioned.
//         // let is_offset_simd = x86_64::_mm256_lddqu_si256(is_offset_ptr as *const x86_64::__m256i);

//         // // Bit level representation of is_offset_simd.
//         // // 0bit -> is_partitioned
//         // // 1bit -> not is_partitioned
//         // let is_offset_packed: i32 = x86_64::_mm256_movemask_epi8(is_offset_simd);

//         // // TODO is that worth it perf wise?
//         // // Efficiently check if all bits are zero.
//         // if is_offset_packed == 0 {
//         //     // All elements are already partitioned.
//         //     block += BLOCK_SIZE;
//         //     continue;
//         // }

//         // let fill = x86_64::_mm256_set1_epi8(-1);

//         // // Test alternative way of writing this.
//         // let indicicies = x86_64::_mm256_set_epi8(
//         //     31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10,
//         //     9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
//         // );

//         // // Scatter values into offsets part.
//         // // Count leading ones in scattered region.
//         // // Copy BLOCK_SIZE into offsets_ptr.
//         // // Update offsets_ptr based on sub_len.

//         // let masked_indicies = x86_64::_mm256_blendv_epi8(fill, indicicies, is_offset_simd);

//         // dest[indices[i]] = src[i]
//         // [0, 0, 0,

//         // We know there will be at least one match because we checked is_offset_packed.
//         let mut scatter_mask = mem::MaybeUninit::<[u8; BLOCK_SIZE]>::uninit();
//         let scatter_mask_ptr = scatter_mask.as_mut_ptr() as *mut u8;

//         let mut x = 0;
//         for i in 0..BLOCK_SIZE {
//             scatter_mask_ptr.add(i).write(x);
//             x += ((*is_offset_ptr.add(i) & 0b10000000u8) != 0) as u8
//         }

//         // _mm256_blendv_epi8

//         // const TEST_INT: i32 = 0b01111101101000001100110110110111i32;;
//         // let x = x86_64::_mm256_permute2f128_si256::<TEST_INT>(zero, indicicies);

//         // println!("{:?}", simd::u8x32::from(offsets_simd).as_array());

//         // let sum_simple = offsets
//         //     .assume_init()
//         //     .iter()
//         //     .map(|x| (*x == u8::MAX) as u8)
//         //     .sum::<u8>();

//         // let x = x86_64::_mm256_movemask_epi8(offsets_simd);
//         // let sum_simd = x.leading_ones() as usize;

//         // sum += sum_simd;

//         // dbg!(sum_simple, sum_simd);

//         // let c = simd::u8x16::from(cmp_result);
//         // let scatter_mask_limited = ptr::slice_from_raw_parts(
//         //     scatter_mask.as_ptr() as *mut u8,
//         //     scatter_mask_ptr.sub_ptr(scatter_mask.as_ptr() as *mut u8),
//         // );

//         // println!("{:?}", is_offset.assume_init_ref());
//         // println!("{:?}", core::simd::u8x32::from(masked_indicies));
//         println!("{:?}", scatter_mask.assume_init_ref());
//         // println!("");

//         block += BLOCK_SIZE;
//     }

//     (offsets, offsets_ptr.sub_ptr(offsets.as_ptr() as *mut u8))
// }

/// Check 128 elements of v and return array of offsets that return true for check(elem, pivot)
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
unsafe fn collect_offsets_128_basic<T, F>(
    v: &[T],
    pivot: &T,
    offsets_base_ptr: *mut u8,
    check: &mut F,
) -> (*mut u8, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    let mut offsets_ptr = offsets_base_ptr;

    // SAFETY: Caller must ensure that v.len() is at least BLOCK_SIZE.
    debug_assert!(v.len() >= N);

    const N: usize = 128;

    // Data hazard, offsets_ptr is read and written each iteration.
    // ~3.2 elem/ns on 5900X
    for i in 0..N {
        offsets_ptr.write(i as u8);
        offsets_ptr = offsets_ptr.add(check(v.get_unchecked(i), pivot) as usize);
    }

    (offsets_ptr, offsets_ptr.sub_ptr(offsets_base_ptr))
}

fn analyze_packed_offset(val: u64) -> (u64, usize) {
    ((val << 3), 1)
}

/// Check 128 elements of v and return array of offsets that return true for check(elem, pivot)
// #[target_feature(enable = "avx2")]
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
unsafe fn collect_offsets_128<T, F>(
    v: &[T],
    pivot: &T,
    offsets_base_ptr: *mut u8,
    check: &mut F,
) -> (*mut u8, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: Caller must ensure that v.len() is at least BLOCK_SIZE.
    debug_assert!(v.len() >= N);

    const BLOCK_SIZE: usize = 32;
    const N: usize = 128;

    let mut block = 0;

    let mut offsets_ptr = offsets_base_ptr;

    while block < N {
        let mut is_offset = mem::MaybeUninit::<[u8; BLOCK_SIZE]>::uninit();
        let mut is_offset_ptr = is_offset.as_mut_ptr() as *mut u8;

        // Avoid data-hazard by not writing into the same pointer each iteration.
        // This should be un-foldable by the optimizer.
        for i in 0..BLOCK_SIZE {
            is_offset_ptr
                .add(i)
                .write(check(v.get_unchecked(block + i), pivot) as u8);
        }

        for i in 0..(BLOCK_SIZE / 16) {
            let is_offset_packed = *((is_offset_ptr as *const u64).add(i));
            let (offset_val, len) = analyze_packed_offset(is_offset_packed);

            (offsets_ptr as *mut u64).write(offset_val);
            offsets_ptr = offsets_ptr.add(len);
        }

        block += BLOCK_SIZE;
    }

    (
        offsets_ptr,
        intrinsics::ptr_offset_from_unsigned(offsets_ptr, offsets_base_ptr),
    )
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    let arr_ptr = v.as_mut_ptr();

    const BLOCK_SIZE: usize = 128;

    if len < BLOCK_SIZE {
        // TODO
        return 0;
    }

    let mut sum_offsets = 0;

    let mut offsets = mem::MaybeUninit::<[u8; BLOCK_SIZE]>::uninit();
    let offsets_base_ptr = offsets.as_mut_ptr() as *mut u8;

    unsafe {
        let mut i = 0;
        while i < len - BLOCK_SIZE {
            let (offsets_ptr, sum) = collect_offsets_128(&v[i..], pivot, offsets_base_ptr, is_less);

            // side effect the indices.
            sum_offsets += (offsets.as_ptr() as *const u8)
                .add(ptr::read_volatile(&0))
                .read_volatile() as usize;

            sum_offsets += sum;

            i += BLOCK_SIZE;
        }
    }

    // dbg!(sum_offsets);

    sum_offsets
}
