//! The idea is to build a partition implementation for types u64 and smaller.

use std::cmp;
use std::mem::{self, MaybeUninit};
use std::ptr;

partition_impl!("butterfly_partition");

/// SAFETY: TODO
#[inline(always)]
unsafe fn partition_up<T, F>(
    elem_ptr: *const T,
    lt_count: usize,
    lt_out_base_ptr: *mut T,
    ge_out_base_ptr: *mut T,
    pivot: &T,
    is_less: &mut F,
) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: See function safety description.
    unsafe {
        let is_lt = is_less(&*elem_ptr, pivot);

        let dest_ptr = if is_lt {
            lt_out_base_ptr
        } else {
            ge_out_base_ptr
        };

        ptr::copy_nonoverlapping(elem_ptr, dest_ptr.add(lt_count), 1);

        lt_count + is_lt as usize
    }
}

/// SAFETY: TODO
#[inline(always)]
unsafe fn partition_down<T, F>(
    elem_ptr: *const T,
    ge_count: usize,
    lt_out_base_ptr: *mut T,
    ge_out_base_ptr: *mut T,
    pivot: &T,
    is_less: &mut F,
) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: See function safety description.
    unsafe {
        let is_lt = is_less(&*elem_ptr, pivot);

        let dest_ptr = if is_lt {
            lt_out_base_ptr
        } else {
            ge_out_base_ptr
        };

        ptr::copy_nonoverlapping(elem_ptr, dest_ptr.add(ge_count), 1);

        ge_count + !is_lt as usize
    }
}

// use std::sync::atomic::{AtomicPtr, Ordering};
// static SCRATCH_LT: AtomicPtr<u64> = AtomicPtr::new(ptr::null_mut());
// static SCRATCH_GE: AtomicPtr<u64> = AtomicPtr::new(ptr::null_mut());

// fn get_scratch<T>(static_ptr: &AtomicPtr<u64>, init_len: usize) -> *mut T {
//     let mut scratch_ptr_u64 = SCRATCH_LT.load(Ordering::Acquire);
//     if scratch_ptr_u64.is_null() {
//         use std::alloc;
//         unsafe {
//             scratch_ptr_u64 =
//                 alloc::alloc(alloc::Layout::array::<u64>(init_len).unwrap()) as *mut u64;
//         }
//         SCRATCH_LT.store(scratch_ptr_u64, Ordering::Release);
//     }
//     assert!(
//         mem::size_of::<T>() <= mem::size_of::<u64>()
//             && mem::align_of::<T>() <= mem::size_of::<u64>()
//     );
//     scratch_ptr_u64 as *mut T
// }

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // TODO T: Freeze

    let len = v.len();
    let arr_ptr = v.as_mut_ptr();

    const BLOCK_SIZE: usize = 256;

    // This is not efficient for other types and large types could cause stack issues.
    // assert!(mem::size_of::<T>() <= mem::size_of::<u64>());

    let mut scratch_lt = MaybeUninit::<[T; BLOCK_SIZE]>::uninit();
    let scratch_lt_ptr = scratch_lt.as_mut_ptr() as *mut T;

    let mut scratch_ge = MaybeUninit::<[T; BLOCK_SIZE]>::uninit();
    let scratch_ge_ptr = scratch_ge.as_mut_ptr() as *mut T;

    // let scratch_lt_ptr = get_scratch::<T>(&SCRATCH_LT, BLOCK_SIZE);
    // let scratch_ge_ptr = get_scratch::<T>(&SCRATCH_GE, BLOCK_SIZE);

    // type DebugT = i32;

    // SAFETY: TODO
    let mut base_ptr = arr_ptr;
    let mut r_ptr = unsafe { arr_ptr.add(len) };

    // SAFETY: TODO
    unsafe {
        loop {
            // TODO intrinsics unlikely.
            // dbg!(i, r_ptr.sub_ptr(arr_ptr));
            let block_size = cmp::min(BLOCK_SIZE, r_ptr.sub_ptr(base_ptr));

            // for i in 0..BLOCK_SIZE {
            //     ptr::copy_nonoverlapping(&999, scratch_lt_ptr.add(i) as *mut DebugT, 1);
            //     ptr::copy_nonoverlapping(&999, scratch_ge_ptr.add(i) as *mut DebugT, 1);
            // }

            let block_size_div_2 = block_size / 2;

            let mut lt_count_up = 0;
            let lt_out_base_ptr_up = scratch_lt_ptr.add(block_size_div_2);
            let mut ge_out_ptr_down = scratch_ge_ptr.add(block_size_div_2);

            let mut ge_count_up = 0;
            let mut lt_out_ptr_down = lt_out_base_ptr_up;
            let ge_out_base_ptr_up = ge_out_ptr_down;

            // Partition grows two buffers independently of each other.
            // Pick mid-point P and grow in both directions <-P-> this allows one contiguous
            // copy for both buffers at the end. Maybe midpoint grow directly into v?
            let mut j = 0;
            while (j + 1) < block_size {
                // Writes lt elements into scratch_lt mid -> up
                // Writes ge elements into scratch_ge down <- mid
                ge_out_ptr_down = ge_out_ptr_down.sub(1);
                lt_count_up = partition_up(
                    base_ptr.add(j),
                    lt_count_up,
                    lt_out_base_ptr_up,
                    ge_out_ptr_down,
                    pivot,
                    is_less,
                );

                // Writes lt elements into scratch_lt down <- mid
                // Writes ge elements into scratch_ge mid -> up
                // TODO invert partition_down logic so that we can use .add on the generated ptr.
                lt_out_ptr_down = lt_out_ptr_down.sub(1);
                ge_count_up = partition_down(
                    base_ptr.add(j + 1),
                    ge_count_up,
                    lt_out_ptr_down,
                    ge_out_base_ptr_up,
                    pivot,
                    is_less,
                );

                j += 2;
            }

            if block_size % 2 != 0 {
                ge_out_ptr_down = ge_out_ptr_down.sub(1);
                lt_count_up = partition_up(
                    base_ptr.add(block_size - 1),
                    lt_count_up,
                    lt_out_base_ptr_up,
                    ge_out_ptr_down,
                    pivot,
                    is_less,
                );
            }

            // println!(
            //     "arr_ptr: {:?}",
            //     &*ptr::slice_from_raw_parts(arr_ptr as *const DebugT, len)
            // );
            // println!(
            //     "scratch_lt_ptr: {:?}",
            //     &*ptr::slice_from_raw_parts(scratch_lt_ptr as *const DebugT, BLOCK_SIZE)
            // );
            // println!(
            //     "scratch_ge_ptr: {:?}",
            //     &*ptr::slice_from_raw_parts(scratch_ge_ptr as *const DebugT, BLOCK_SIZE)
            // );

            // dbg!(block_size_div_2, ge_count_up);
            let lt_count_down = block_size_div_2 - ge_count_up;
            let lt_count = lt_count_up + lt_count_down;
            let ge_count = block_size - lt_count;
            let orig_base_ptr = base_ptr;
            // dbg!(lt_count_up, ge_count_up, lt_count, ge_count);

            // let base_diff = base_ptr.sub_ptr(arr_ptr);
            // println!("base now: {} -> {}", base_diff, base_diff + lt_count);

            base_ptr = base_ptr.add(lt_count);
            let orig_r_ptr = r_ptr;
            r_ptr = r_ptr.sub(ge_count);

            // Copy elements from the right side that will be overwritten by ge elements.
            if block_size == BLOCK_SIZE {
                // Only necessary if there will be future blocks that we look at.
                // Otherwise the two scratch buffers hold all the necessary information.
                let save_count = cmp::min(ge_count, r_ptr.sub_ptr(base_ptr));
                ptr::copy_nonoverlapping(orig_r_ptr.sub(save_count), base_ptr, save_count);
            }

            // Copy the less than (lt) elements to the start of base_ptr.
            // let x = lt_out_base_ptr_up.sub(lt_count_down);
            // let base_diff = base_ptr.sub_ptr(arr_ptr);
            // assert!(
            //     orig_base_ptr.add(lt_count) <= arr_ptr.add(len) && x >= scratch_lt_ptr,
            //     "{len} {base_diff} {lt_count} arr_ptr: {:?} pivot: {}",
            //     &*ptr::slice_from_raw_parts(arr_ptr as *const DebugT, len),
            //     *(pivot as *const T as *const DebugT)
            // );
            // for i in 0..lt_count {
            //     assert!(
            //         orig_base_ptr.add(i) < arr_ptr.add(len),
            //         "len: {len} base_diff: {base_diff} lt_count: {lt_count} arr_ptr: {:?} pivot: {}",
            //         &*ptr::slice_from_raw_parts(arr_ptr as *const DebugT, len),
            //         *(pivot as *const T as *const DebugT)
            //     );
            //     ptr::copy_nonoverlapping(x.add(i), orig_base_ptr.add(i), 1);
            // }

            ptr::copy_nonoverlapping(
                lt_out_base_ptr_up.sub(lt_count_down),
                orig_base_ptr,
                lt_count,
            );

            // Copy the greater or equal (ge) elements to the right side.
            ptr::copy_nonoverlapping(ge_out_ptr_down.add(lt_count_up), r_ptr, ge_count);

            // println!(
            //     "arr_ptr after: {:?}",
            //     &*ptr::slice_from_raw_parts(arr_ptr as *const DebugT, len)
            // );

            // Instead of swapping between processing elements on the left and then on the right.
            // Copy elements from the right and keep processing from the left. This greatly reduces
            // code-gen. And allows to use a variable size block and larger sizes to amortize the
            // cost of calling memcpy.

            if base_ptr >= r_ptr {
                break;
            }
        }

        base_ptr.sub_ptr(arr_ptr)
    }
}
