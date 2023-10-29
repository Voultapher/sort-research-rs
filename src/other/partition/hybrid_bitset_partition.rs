//! See comment in [`partition`].

use core::cmp;
use core::mem::{self, MaybeUninit};
use core::ptr;

partition_impl!("hybrid_bitset_partition");

// Can the type have interior mutability, this is checked by testing if T is Freeze. If the type can
// have interior mutability it may alter itself during comparison in a way that must be observed
// after the sort operation concludes. Otherwise a type like Mutex<Option<Box<str>>> could lead to
// double free.
unsafe auto trait Freeze {}

impl<T: ?Sized> !Freeze for core::cell::UnsafeCell<T> {}
unsafe impl<T: ?Sized> Freeze for core::marker::PhantomData<T> {}
unsafe impl<T: ?Sized> Freeze for *const T {}
unsafe impl<T: ?Sized> Freeze for *mut T {}
unsafe impl<T: ?Sized> Freeze for &T {}
unsafe impl<T: ?Sized> Freeze for &mut T {}

// This number is chosen to strike a balance between good perf handoff between only small_partition
// and hybrid block_partition + small_partition. Avoiding stack buffers that are too large,
// impacting efficiency negatively. And aromatizing the cost of runtime feature detection, mostly a
// relaxed atomic load + non-inlined function call.
const MAX_SMALL_PARTITION_LEN: usize = 255;

// TODO not pub and debug
pub struct BlockPartitionResult {
    pub lt_count: usize,
    pub un_partitioned_count: usize,
    pub l_bitmap: BitsetStorageT,
    pub r_bitmap: BitsetStorageT,
}

trait Partition: Sized {
    /// Takes the input slice `v` and re-arranges elements such that when the call returns normally
    /// all elements that compare true for `is_less(elem, pivot)` are on the left side of `v`
    /// followed by the other elements, notionally considered greater or equal to `pivot`.
    ///
    /// Returns the number of elements that are compared true for `is_less(elem, pivot)`.
    ///
    /// If `is_less` does not implement a total order the resulting order and return value are
    /// unspecified. All original elements will remain in `v` and any possible modifications via
    /// interior mutability will be observable. Same is true if `is_less` panics or `v.len()`
    /// exceeds [`MAX_SMALL_PARTITION_LEN`].
    fn small_partition<F>(v: &mut [Self], pivot: &Self, is_less: &mut F) -> usize
    where
        F: FnMut(&Self, &Self) -> bool;

    /// Takes the input slice `v` and re-arranges elements such that when the call returns normally
    /// most elements that compare true for `is_less(elem, pivot)` are on the left side of `v`
    /// followed by an area that is yet to be partitioned, followed by elements that compared false,
    /// notionally considered greater or equal to `pivot`.
    ///
    /// Returns [`BlockPartitionResult`] where `left_ptr` points to the first element inside `v`
    /// that is not part of the un-partitioned elements and `un_partitioned_count` describes the
    /// length of said region, which may be zero. And `l_bitmap` and `r_bitmap` are the bitmaps
    /// where one of them might contain unused comparison results.
    ///
    /// If `is_less` does not implement a total order the resulting order and return value are
    /// unspecified but still guaranteed `left_ptr >= arr_ptr && left_ptr.add(un_partition_len) <=
    /// arr_ptr.add(len)`. All original elements will remain in `v` and any possible modifications
    /// via interior mutability will be observable. Same is true if `is_less` panics.
    fn block_partition<F>(v: &mut [Self], pivot: &Self, is_less: &mut F) -> BlockPartitionResult
    where
        F: FnMut(&Self, &Self) -> bool;
}

/// SAFETY: The caller must ensure that all provided expression are no-panic and may not modify the
/// values produced by `next_left` and `next_right`. And the produced pointers MUST NOT alias.
macro_rules! cyclic_permutation_swap_loop {
    ($continue_check:expr, $next_left:expr, $next_right:expr) => {
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

        if $continue_check {
            let mut left_ptr = $next_left;
            let mut right_ptr = $next_right;

            // SAFETY: The following code is both panic- and observation-safe, so it's ok to
            // create a temporary.
            let tmp = ptr::read(left_ptr);
            ptr::copy_nonoverlapping(right_ptr, left_ptr, 1);

            while $continue_check {
                left_ptr = $next_left;
                ptr::copy_nonoverlapping(left_ptr, right_ptr, 1);
                right_ptr = $next_right;
                ptr::copy_nonoverlapping(right_ptr, left_ptr, 1);
            }

            ptr::copy_nonoverlapping(&tmp, right_ptr, 1);
            mem::forget(tmp);
        }
    };
}

macro_rules! instantiate_block_partition {
    ($fn_name:ident $(#[$StructMeta:meta])*) => {
        /// See [`Partition::block_partition`].
        $(#[$StructMeta])*
        unsafe fn $fn_name<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> BlockPartitionResult
        where
            F: FnMut(&T, &T) -> bool,
        {
            const BLOCK: usize = BLOCK_PARTITION_BLOCK_SIZE;

            let len = v.len();
            let arr_ptr = v.as_mut_ptr();

            // lt == less than, ge == greater or equal
            let mut l_bitmap: BitsetStorageT = 0; // aka ge_bitmap
            let mut r_bitmap: BitsetStorageT = 0; // aka lt_bitmap

            if len < (2 * BLOCK) {
                debug_assert!(false);
                return BlockPartitionResult {
                    lt_count: 0,
                    un_partitioned_count: len,
                    l_bitmap,
                    r_bitmap,
                };
            }

            // SAFETY: TODO
            unsafe {
                let mut l_ptr = arr_ptr;
                let mut r_ptr = arr_ptr.add(len - BLOCK);

                // It's crucial for reliable auto-vectorization that BLOCK always stays the same. Which
                // means we handle the rest of the input size separately later.

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

                    // TODO try out version that generates masks based on i.
                    // TODO try out version that is manually unrolled to two.
                    cyclic_permutation_swap_loop!(
                        {
                            // continue_check
                            l_bitmap > 0 && r_bitmap > 0
                        },
                        {
                            // next_left
                            let l_idx = l_bitmap.trailing_zeros() as usize;
                            l_bitmap = clear_lowest_bit(l_bitmap);
                            l_ptr.add(l_idx)
                        },
                        {
                            // next_right
                            let r_idx = r_bitmap.trailing_zeros() as usize;
                            r_bitmap = clear_lowest_bit(r_bitmap);
                            r_ptr.add(r_idx)
                        }
                    );

                    l_ptr = l_ptr.add((l_bitmap == 0) as usize * BLOCK);
                    r_ptr = r_ptr.sub((r_bitmap == 0) as usize * BLOCK);
                    // println!(
                    //     "l_ptr: {} r_ptr: {}",
                    //     l_ptr.sub_ptr(arr_ptr),
                    //     r_ptr.sub_ptr(arr_ptr)
                    // );
                }

                let r_end_ptr = r_ptr.add(BLOCK);

                BlockPartitionResult {
                    lt_count: l_ptr.sub_ptr(arr_ptr),
                    un_partitioned_count: r_end_ptr.sub_ptr(l_ptr),
                    l_bitmap,
                    r_bitmap,
                }
            }
        }
    };
}

instantiate_block_partition!(block_partition);

impl<T> Partition for T {
    default fn small_partition<F>(v: &mut [Self], pivot: &Self, is_less: &mut F) -> usize
    where
        F: FnMut(&Self, &Self) -> bool,
    {
        small_partition_move_opt(v, pivot, is_less)
    }

    // #[inline(never)] // TODO check that.
    default fn block_partition<F>(
        v: &mut [Self],
        pivot: &Self,
        is_less: &mut F,
    ) -> BlockPartitionResult
    where
        F: FnMut(&Self, &Self) -> bool,
    {
        unsafe { block_partition(v, pivot, is_less) }
    }
}

impl<T: Freeze + Copy> Partition for T {
    fn small_partition<F>(v: &mut [Self], pivot: &Self, is_less: &mut F) -> usize
    where
        F: FnMut(&Self, &Self) -> bool,
    {
        if const { mem::size_of::<T>() <= mem::size_of::<[usize; 2]>() } {
            small_partition_int_opt(v, pivot, is_less)
        } else {
            small_partition_move_opt(v, pivot, is_less)
        }
    }

    fn block_partition<F>(v: &mut [Self], pivot: &Self, is_less: &mut F) -> BlockPartitionResult
    where
        F: FnMut(&Self, &Self) -> bool,
    {
        if const { mem::size_of::<T>() <= mem::size_of::<usize>() } {
            // TODO feature detection.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                instantiate_block_partition!(block_partition_vectorized #[target_feature(enable = "avx")]);

                if std::is_x86_feature_detected!("avx") {
                    // SAFETY: We checked that the feature is supported by the CPU.
                    unsafe {
                        return block_partition_vectorized(v, pivot, is_less);
                    }
                }
            }
        }

        unsafe { block_partition(v, pivot, is_less) }
    }
}

/// See [`Partition::small_partition`].
///
/// Optimized for integers like types. Not suitable for large types, because it stores temporary
/// copies in a stack buffer.
fn small_partition_int_opt<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
    T: Freeze,
{
    let len = v.len();
    let arr_ptr = v.as_mut_ptr();

    if len > MAX_SMALL_PARTITION_LEN {
        debug_assert!(false);
        return 0;
    }

    // SAFETY: TODO
    unsafe {
        let mut scratch = MaybeUninit::<[T; MAX_SMALL_PARTITION_LEN]>::uninit();
        let scratch_ptr = scratch.as_mut_ptr() as *mut T;

        let mut lt_count = 0;
        let mut ge_out_ptr = scratch_ptr.add(len);

        // Loop manually unrolled to ensure good performance.
        // Example T == u64, on x86 LLVM unrolls this loop but not on Arm.
        // And it's very perf critical so this is done manually.
        // And surprisingly this can yield better code-gen and perf than the auto-unroll.
        macro_rules! loop_body {
            ($elem_ptr:expr) => {
                ge_out_ptr = ge_out_ptr.sub(1);

                let elem_ptr = $elem_ptr;

                let is_less_than_pivot = is_less(&*elem_ptr, pivot);

                // Benchmarks show that especially on Firestorm (apple-m1) for anything at
                // most the size of a u64 double storing is more efficient than conditional
                // store. It is also less at risk of having the compiler generating a branch
                // instead of conditional store.
                if const { mem::size_of::<T>() <= mem::size_of::<usize>() } {
                    ptr::copy_nonoverlapping(elem_ptr, scratch_ptr.add(lt_count), 1);
                    ptr::copy_nonoverlapping(elem_ptr, ge_out_ptr.add(lt_count), 1);
                } else {
                    let dest_ptr = if is_less_than_pivot {
                        scratch_ptr
                    } else {
                        ge_out_ptr
                    };
                    ptr::copy_nonoverlapping(elem_ptr, dest_ptr.add(lt_count), 1);
                }

                lt_count += is_less_than_pivot as usize;
            };
        }

        let mut i: usize = 0;
        let end = len.saturating_sub(1);

        while i < end {
            loop_body!(arr_ptr.add(i));
            loop_body!(arr_ptr.add(i + 1));
            i += 2;
        }

        if i != len {
            loop_body!(arr_ptr.add(i));
        }

        // SAFETY: swap now contains all elements that belong on the left side of the pivot.
        // All comparisons have been done if is_less would have panicked `v` would have
        // stayed untouched.
        ptr::copy_nonoverlapping(scratch_ptr, arr_ptr, len);

        lt_count
    }
}

/// See [`Partition::small_partition`].
///
/// Optimized for minimal moves.
fn small_partition_move_opt<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    let arr_ptr = v.as_mut_ptr();

    if len > MAX_SMALL_PARTITION_LEN {
        debug_assert!(false);
        return 0;
    }

    // Larger types are optimized for a minimal amount of moves and avoid stack arrays with a size
    // dependent on T. It's not crazy fast for something like `u64`, still 2x faster than a simple
    // branchy version. But for things like `String` it's as fast if not faster and it saves on
    // compile-time to only instantiate the other version for types that are likely to benefit.

    // SAFETY: TODO
    unsafe {
        let mut ge_idx_buffer = MaybeUninit::<[u8; MAX_SMALL_PARTITION_LEN]>::uninit();
        let ge_idx_ptr = ge_idx_buffer.as_mut_ptr() as *mut u8;

        let mut lt_idx_buffer = MaybeUninit::<[u8; MAX_SMALL_PARTITION_LEN]>::uninit();
        let mut lt_idx_ptr = (lt_idx_buffer.as_mut_ptr() as *mut u8).add(len);

        let mut ge_count = 0;

        for i in 0..len {
            lt_idx_ptr = lt_idx_ptr.sub(1);

            *ge_idx_ptr.add(ge_count) = i as u8;
            *lt_idx_ptr.add(ge_count) = i as u8;

            let is_ge = !is_less(&*arr_ptr.add(i), pivot);
            ge_count += is_ge as usize;
        }

        let lt_count = len - ge_count;
        lt_idx_ptr = lt_idx_ptr.add(ge_count);

        let mut i = usize::MAX;
        cyclic_permutation_swap_loop!(
            {
                // continue_check
                i = i.wrapping_add(1);
                i < lt_count && (*ge_idx_ptr.add(i) as usize) < lt_count
            },
            {
                // next_left
                arr_ptr.add(*ge_idx_ptr.add(i) as usize)
            },
            {
                // next_right
                arr_ptr.add(*lt_idx_ptr.add(i) as usize)
            }
        );

        lt_count
    }
}

// Using 32 bits as bitset and with that as block-size has various benefits. It nicely unrolls the
// inner pivot comparison loop into a single block of SIMD instructions and it doesn't tempt the
// prefetcher into fetching too much info on the right side. This is with `u64` as the largest type
// expected to greatly benefit from vectorization.
type BitsetStorageT = u32;

/// Scan elements `base_ptr[..block_len]` and build a bitset that has the corresponding bit toggled
/// depending on `is_swap_elem`.
///
/// Written in a way that enables reliable auto-vectorization by the compiler if wide enough SIMD is
/// available.
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

// TODO remove
// #[inline(always)]
// fn clear_highest_bit(x: BitsetStorageT) -> BitsetStorageT {
//     x ^ (1 << ((BLOCK_PARTITION_BLOCK_SIZE - 1) - x.leading_zeros()))
// }

// TODO explain more. Both AVX and NEON SIMD were analyzed for `u64` and `i32` element types,
// the inner pivot comparison loop should spend a bit less than a cycle per element doing the
// comparison and 1.5-2.5 cycles if no SIMD is available. TODO cycles per swapped elements.
const BLOCK_PARTITION_BLOCK_SIZE: usize = BitsetStorageT::BITS as usize;
const BITSET_ALL_SET: BitsetStorageT = BitsetStorageT::MAX;

/// Takes the slice `v` and the `block_partition_result` which may contain one bitmap with unused
/// comparison results, and modifies `v` to shrink the un-partitioned gap down as much as possible.
/// If the left bitmap has leftovers the lt elements on the left side are shuffled such that they
/// are continuous on the left side, the ge elements from the left bitmap put on the right side and
/// the left gap filled with unknown elements from the un-partitioned area, vice versa if the right
/// bitmap contains leftovers.
///
/// Returns a slice that covers the remaining un-partitioned area.
// TODO not pub
pub fn use_bitmap_info<T>(v: &mut [T], block_partition_result: BlockPartitionResult) -> &mut [T] {
    // This logic is pulled out of `block_partition` to avoid duplicate instantiation. It's
    // possible to forward the `block_partition_result` into `small_partition` but that makes
    // the code *a lot* harder to reason about and adds more branches to the hot path of no
    // block partitioning at all. It's also possible to ignore the info left in the bitmaps and
    // re-do the comparisons as part of `small_partition`. But that's not a good fit for a
    // generic implementation and has the surprising property that `partition` may do more than
    // `v.len()` comparisons.
    const BLOCK: usize = BLOCK_PARTITION_BLOCK_SIZE;

    // type DebugT = i32;
    // print(unsafe { mem::transmute::<&[T], &[DebugT]>(v) });
    // print(block_partition_result);

    if v.len() < (2 * BLOCK) {
        debug_assert!(false); // Logic bug.
        return v;
    }

    // SAFETY: TODO
    unsafe {
        let l_ptr = v.as_mut_ptr().add(block_partition_result.lt_count);

        let mut l_adjusted_ptr = l_ptr;
        let un_partitioned_count = block_partition_result.un_partitioned_count;

        let l_bitmap = block_partition_result.l_bitmap;
        let r_bitmap = block_partition_result.r_bitmap;

        // It would be a logic bug if somehow cyclic_permutation_swap_loop left both blocks with
        // remaining elements, or the remaining area is less than a block large.
        debug_assert!(!(l_bitmap != 0 && r_bitmap != 0));

        if false && l_bitmap | r_bitmap != 0 {
            // TODO remove
            debug_assert!(block_partition_result.un_partitioned_count >= BLOCK);

            // type DebugT = i32;
            // println!("");
            // println!(
            //     "l_block_area:        {:?}",
            //     &*ptr::slice_from_raw_parts(l_adjusted_ptr as *const DebugT, BLOCK)
            // );
            // println!(
            //     "r_block_area:        {:?}",
            //     &*ptr::slice_from_raw_parts(
            //         l_ptr.add(un_partitioned_count - BLOCK) as *const DebugT,
            //         BLOCK
            //     )
            // );
            // println!(
            //     "un_partitioned_area: {:?}",
            //     &*ptr::slice_from_raw_parts(l_adjusted_ptr as *const DebugT, un_partitioned_count)
            // );

            let is_l_bitmap = l_bitmap != 0;

            let (mut bitmap, block_start_ptr) = if is_l_bitmap {
                (l_bitmap, l_ptr)
            } else {
                // Invert the right bitmap to move elements to the left instead of the right in the
                // loop below.
                (
                    r_bitmap ^ BITSET_ALL_SET,
                    l_ptr.add(un_partitioned_count - BLOCK),
                )
            };

            // println!("bitmap: 0b{bitmap:032b}");
            if bitmap != BITSET_ALL_SET {
                // Kind of a mini Hoare partition within the unfinished bitset region.
                loop {
                    // TODO check if these are needed.
                    core::intrinsics::assume(bitmap != 0);
                    core::intrinsics::assume(bitmap != BITSET_ALL_SET);

                    let l = bitmap.trailing_zeros() as usize;
                    let r = (BLOCK - 1) - bitmap.leading_ones() as usize;

                    if l >= r {
                        break;
                    }

                    ptr::swap_nonoverlapping(block_start_ptr.add(l), block_start_ptr.add(r), 1);

                    bitmap = clear_lowest_bit(bitmap);
                    bitmap |= 1 << r; // set specific bit.
                }
            }

            // println!(
            //     "r_block_area:n       {:?}",
            //     &*ptr::slice_from_raw_parts(
            //         l_ptr.add(un_partitioned_count - BLOCK) as *const DebugT,
            //         BLOCK
            //     )
            // );

            // println!(
            //     "un_partitioned_area:s{:?}",
            //     &*ptr::slice_from_raw_parts(l_ptr as *const DebugT, un_partitioned_count)
            // );

            // How many elements were shuffled in such a way that they now join either the left or
            // right side.
            let left_shuffle_count = bitmap.trailing_zeros() as usize;
            let right_shuffle_count = BLOCK - left_shuffle_count;
            let new_un_partitioned_count = un_partitioned_count - BLOCK;

            // dbg!(left_shuffle_count);
            // dbg!(right_shuffle_count);
            // dbg!(un_partitioned_count);
            // dbg!(new_un_partitioned_count);

            // println!("l_adjusted_ptr: {}", l_adjusted_ptr.sub_ptr(l_ptr));

            let swap_count;
            let l_swap_ptr;
            let r_swap_ptr;

            if is_l_bitmap {
                swap_count = cmp::min(right_shuffle_count, new_un_partitioned_count);
                l_swap_ptr = l_ptr.add(left_shuffle_count);
                r_swap_ptr = l_ptr.add(un_partitioned_count - swap_count);
                l_adjusted_ptr = l_swap_ptr;
            } else {
                swap_count = cmp::min(left_shuffle_count, new_un_partitioned_count);
                l_swap_ptr = l_ptr;
                r_swap_ptr = l_ptr.add(un_partitioned_count - (right_shuffle_count + swap_count));

                // TODO there has to be a neater way of writing all this.
                // TODO also check n comp promise for partition.
                if swap_count > left_shuffle_count {
                    l_adjusted_ptr = l_swap_ptr.add(swap_count);
                }
            }

            // dbg!(swap_count);

            // println!(
            //     "l_swap_ptr val: {} r_swap_ptr val: {} l_swap_ptr offset: {} r_swap_ptr offset: {}, swap_count: {}",
            //     *(l_swap_ptr as *const DebugT),
            //     *(r_swap_ptr as *const DebugT),
            //     l_swap_ptr.sub_ptr(v.as_ptr()),
            //     r_swap_ptr.sub_ptr(v.as_ptr()),
            //     swap_count
            // );

            ptr::swap_nonoverlapping(l_swap_ptr, r_swap_ptr, swap_count);

            // println!(
            //     "un_partitioned_area:o{:?}",
            //     &*ptr::slice_from_raw_parts(l_ptr as *const DebugT, un_partitioned_count)
            // );
            // println!(
            //     "un_partitioned_area:n{:?}",
            //     &*ptr::slice_from_raw_parts(l_adjusted_ptr as *const DebugT, un_partitioned_count)
            // );
        }

        // {
        //     // TODO remove
        //     let new_start_ptr = l_adjusted_ptr.add(un_partitioned_count) as *const T;
        //     println!(
        //         "v.len(): {} new_start_ptr offset: {} l_ptr offset: {}",
        //         v.len(),
        //         new_start_ptr.sub_ptr(v.as_ptr()),
        //         l_ptr.sub_ptr(v.as_ptr())
        //     );

        //     debug_assert!(new_start_ptr >= v.as_ptr());
        //     debug_assert!(new_start_ptr <= v.as_ptr().add(v.len()));
        // }

        &mut *ptr::slice_from_raw_parts_mut(l_adjusted_ptr, un_partitioned_count)
    }

    // // SAFETY: The implementation of `block_partition` must be valid and uphold the contract
    // // specified in the trait documentation.
    // let (lt_block_count, small_partition_slice) = unsafe {
    //     (
    //         left_ptr.sub_ptr(arr_ptr),
    //         &mut *ptr::slice_from_raw_parts_mut(left_ptr, small_partition_len),
    //     )
    // };
}

/// Takes the input slice `v` and re-arranges elements such that when the call returns normally
/// all elements that compare true for `is_less(elem, pivot)` are on the left side of `v`
/// followed by the other elements, notionally considered greater or equal to `pivot`.
///
/// Returns the number of elements that are compared true for `is_less(elem, pivot)`.
///
/// If `is_less` does not implement a total order the resulting order and return value are
/// unspecified. All original elements will remain in `v` and any possible modifications via
/// interior mutability will be observable. Same is true if `is_less` panics.
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F: FnMut(&T, &T) -> bool>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize {
    // This partition implementation combines various ideas to strike a good balance optimizing for
    // all the following:
    //
    // - performance/efficiency
    // - compile-time
    // - binary-size
    // - wide range of input lengths
    // - diverse types (integers, Strings, big stack arrays, etc.)
    // - various ISAs (x86, Arm, RISC-V, etc.)

    // High level overview and motivation:
    //
    // There are two main components, a small_partition implementation that is optimized for small
    // input sizes and can only handle up to `MAX_SMALL_PARTITION_LEN` elements. A block_partition
    // implementation optimized for consistent high throughput for larger sizes that may leave a
    // small region in the middle of the input slice un-partitioned. Either the input slice length
    // is small enough to be handled entirely by the small_partition, or it first handles most of
    // the input with the block_partition and the remaining hole with the small_partition. This
    // allows both components to be specialized and limits binary-size as well as branching overhead
    // to handle various scenarios commonly involved when handling the remainder of some block based
    // partition scheme. This scheme also allows the block_partition to use runtime feature
    // detection to leverage SIMD to speed up fixed block size processing, while only having to
    // double instantiate the block processing part and not the remainder handling which doesn't
    // benefit from it. Further, only calling block_partition for larger input length amortizes the
    // cost of runtime feature detection and last block handling. The implementations use heuristics
    // based on properties like input type size as well as Freeze and Copy traits to choose between
    // implementation strategies as appropriate, this can be seen as a form of type introspection
    // based programming. Using a block based partition scheme combined with a cyclic permutation is
    // a good fit for generic Rust implementation because it's trivial to prove panic- and
    // observation-safe as it disconnects, calling the user-provided comparison function which may
    // panic and or modify the values that are being compared, with creating temporary copies. In
    // addition using a cyclic permutation and only swapping values that need to be swapped is
    // efficient for both small types like integers and arbitrarily large user-defined types, as
    // well as cases where the input is already fully or nearly partitioned as may happen when
    // filtering out common values in a pdqsort style equal partition.

    // Influences:
    //
    // Many of the component ideas at play here are not novel and some were researched and
    // discovered independently to prior art.
    //
    // block_partition is fundamentally a Hoare partition, which Stefan Edelkamp and Armin Weiß used
    // in their paper "BlockQuicksort: How Branch Mispredictions don’t affect Quicksort"
    // [https://arxiv.org/pdf/1604.06697.pdf] and added unrolled block level processing, branchless
    // offset generation and cyclic permutation based swapping. Orson Peters used this in his paper
    // "Pattern-defeating Quicksort" [https://arxiv.org/pdf/2106.05123.pdf] and refined the
    // code-gen. The work on pdqsort was then used as the starting point for the work done by
    // Min-Jae Hwang in Bitset Sort [https://github.com/minjaehwang/bitsetsort] which changes the
    // block offset calculation in a way that allows for reliable compiler auto-vectorization, but
    // requires wider SIMD to be available than the default x86 and Arm LLVM targets include by
    // default, to benefit from auto-vectorization. This then formed the basis for the work by Lukas
    // Bergdoll on this block_partition which refines code-gen and adds double instantiation, one
    // default one and one with for example x86 AVX code-gen enabled paired with type introspection
    // heuristics to avoid generating the double instantiation for types like String which will not
    // auto-vectorize anyway. As well as a way to avoid the double instantiation and runtime feature
    // detection entirely if compiled with flags that allow wide enough SIMD anyway, allowing for a
    // kind of static dispatch.
    //
    // small_partition is two entirely different partition implementations that use type
    // introspection to choose between them at compile time. One is focused on integer like types
    // and is based on research in sort-research-rs
    // [https://github.com/Voultapher/sort-research-rs/blob/c9f5ce28ff5705f119e0fab0626792304f36eecd/src/other/partition/small_fast.rs]
    // later refined in driftsort by Orson Peters and Lukas Bergdoll [TODO link]. The other version
    // focused on minimal moves is a novel design by the author that does a single scan with
    // code-gen influenced by driftsort followed by a cyclic permutation with an early exit, doing
    // the bare minimum moves.

    let len = v.len();
    let arr_ptr = v.as_ptr();

    let mut lt_block_count = 0;
    let mut local_v = v;

    if len > MAX_SMALL_PARTITION_LEN {
        let block_partition_result = <T as Partition>::block_partition(local_v, pivot, is_less);
        local_v = use_bitmap_info(local_v, block_partition_result);
        // SAFETY: block_partition and use_bitmap_info are expected to work correctly and use_bitmap_info is expected to return a slice that is within `v`.
        lt_block_count = unsafe { local_v.as_ptr().sub_ptr(arr_ptr) };
    }

    lt_block_count + <T as Partition>::small_partition(local_v, pivot, is_less)
}
