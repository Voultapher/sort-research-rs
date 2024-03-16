use std::alloc;
use std::intrinsics;
use std::mem::{self, MaybeUninit};
use std::ptr;
use std::slice;

use crate::other::partition::get_or_alloc_tls_scratch;

partition_impl!("stable_2side_fill");

#[must_use]
const fn is_int_like_type<T>() -> bool {
    // A heuristic that guesses whether a type looks like an int for optimization purposes.
    /*<T as IsFreeze>::value() &&*/
    mem::size_of::<T>() <= mem::size_of::<u64>()
}

/// Takes the input slice `v` and re-arranges elements such that when the call returns normally all
/// elements that compare true for `is_less(elem, pivot)` where `pivot == v[pivot_pos]` are on the
/// left side of `v` followed by the other elements, notionally considered greater or equal to
/// `pivot`.
///
/// Returns the number of elements that are compared true for `is_less(elem, pivot)`.
///
/// If `is_less` does not implement a total order the resulting order and return value are
/// unspecified. All original elements will remain in `v` and any possible modifications via
/// interior mutability will be observable. Same is true if `is_less` panics or `v.len()` exceeds
/// `scratch.len()`.
fn stable_partition<T, F>(
    v: &mut [T],
    scratch: &mut [MaybeUninit<T>],
    pivot: &T,
    is_less: &mut F,
) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    let arr_ptr = v.as_mut_ptr();

    if intrinsics::unlikely(scratch.len() < len) {
        debug_assert!(false); // That's a logic bug in the implementation.
        return 0;
    }

    // Inside the main partitioning loop we MUST NOT compare our stack copy of the pivot value with
    // the original value in the slice `v`. If we just write the value as pointed to by `src_ptr`
    // into `sctratch_ptr` as it was in the input slice `v` we would risk that the call to the
    // user-provided `is_less` modifies the value pointed to by `src_ptr`. This could be UB for
    // types such as `Mutex<Option<Box<String>>>` where during the comparison it replaces the box
    // with None, leading to double free. As the value written back into `v` from `sctratch_ptr` did
    // not observe that modification.
    //
    // Partitioning loop manually unrolled to ensure good performance. Example T == u64, on x86 LLVM
    // unrolls this loop but not on Arm. A compile time fixed size loop as based on `unroll_len` is
    // reliably unrolled by all backends. And if `unroll_len` is `1` the inner loop can trivially be
    // removed.
    //
    // The scheme used to unroll is somewhat weird, and focused on avoiding multi-instantiation of
    // the inner loop part, which can have large effects on compile-time for non integer like types.

    // SAFETY: TODO
    unsafe {
        let scratch_ptr = MaybeUninit::slice_as_mut_ptr(scratch);

        // lt == less than, ge == greater or equal
        let mut lt_count = 0;
        let mut ge_out_ptr = scratch_ptr.add(len);

        let unroll_len = if const { is_int_like_type::<T>() } {
            8
        } else {
            1 // If the optimizer is convinced it is useful, it can still unroll this case.
        };

        let mut base_i = 0;
        'outer: loop {
            for unroll_i in 0..unroll_len {
                let i = base_i + unroll_i;
                if intrinsics::unlikely(i >= len) {
                    break 'outer;
                }
                let elem_ptr = arr_ptr.add(i);

                ge_out_ptr = ge_out_ptr.sub(1);

                // This is required to
                // handle types with interior mutability. See comment above for more info.

                let is_less_than_pivot = is_less(&*elem_ptr, pivot);

                if const { mem::size_of::<T>() <= mem::size_of::<u64>() } {
                    // Benchmarks show that especially on Firestorm (apple-m1) for anything at
                    // most the size of a u64, double storing is more efficient than conditional
                    // store. It is also less at risk of having the compiler generating a branch
                    // instead of conditional store.
                    ptr::copy_nonoverlapping(elem_ptr, scratch_ptr.add(lt_count), 1);
                    ptr::copy_nonoverlapping(elem_ptr, ge_out_ptr.add(lt_count), 1);
                } else {
                    let dst_ptr = if is_less_than_pivot {
                        scratch_ptr
                    } else {
                        ge_out_ptr
                    };
                    ptr::copy_nonoverlapping(elem_ptr, dst_ptr.add(lt_count), 1);
                }

                lt_count += is_less_than_pivot as usize;
            }

            base_i += unroll_len;
        }

        // Copy all the elements that were not equal directly from swap to v.
        ptr::copy_nonoverlapping(scratch_ptr, arr_ptr, lt_count);

        // Copy the elements that were equal or more from the buf into v and reverse them.
        let rev_buf_ptr = scratch_ptr.add(len - 1);
        for i in 0..len - lt_count {
            ptr::copy_nonoverlapping(rev_buf_ptr.sub(i), arr_ptr.add(lt_count + i), 1);
        }

        lt_count
    }
}

fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    let scratch_ptr = get_or_alloc_tls_scratch(alloc::Layout::array::<T>(len).unwrap());
    let scratch =
        unsafe { slice::from_raw_parts_mut(scratch_ptr.as_ptr() as *mut MaybeUninit<T>, len) };

    stable_partition(v, scratch, pivot, is_less)
}
