use core::mem::{self, MaybeUninit};
use core::ptr;

partition_impl!("small_partition");

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

const MAX_SMALL_PARTITION_LEN: usize = 128;

trait SmallPartitionImpl: Sized {
    fn partition<F>(v: &mut [Self], pivot: &Self, is_less: &mut F) -> usize
    where
        F: FnMut(&Self, &Self) -> bool;
}

fn small_partition_default<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    if len > MAX_SMALL_PARTITION_LEN {
        debug_assert!(false);
        return 0;
    }

    let arr_ptr = v.as_mut_ptr();

    // Larger types are optimized for a minimal amount of moves and avoid stack arrays with a size
    // dependent on T. It's not crazy fast for something like `u64`, still 2x faster than a simple
    // branchy version. But for things like `String` it's as fast if not faster and it saves on
    // compile-time to only instantiate the other version for types that are likely to benefit.

    // SAFETY: TODO
    unsafe {
        let mut ge_idx_buffer = MaybeUninit::<[u8; MAX_SMALL_PARTITION_LEN]>::uninit();
        let ge_idx_ptr = ge_idx_buffer.as_mut_ptr() as *mut u8;

        let mut lt_idx_buffer = MaybeUninit::<[u8; MAX_SMALL_PARTITION_LEN]>::uninit();
        let mut lt_idx_ptr = lt_idx_buffer.as_mut_ptr() as *mut u8;
        lt_idx_ptr = lt_idx_ptr.add(len);

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

        macro_rules! left_idx {
            ($i:expr) => {
                *ge_idx_ptr.add($i) as usize
            };
        }

        macro_rules! right_idx {
            ($i:expr) => {
                *lt_idx_ptr.add($i) as usize
            };
        }

        // This is a  cyclic permutation that does on average 2 moves per swap
        // instead of 3 when using `ptr::swap_nonoverlapping`.
        if lt_count >= 1 && left_idx!(0) < lt_count {
            let mut left_ptr = arr_ptr.add(left_idx!(0));
            let mut right_ptr = arr_ptr.add(right_idx!(0));

            // SAFETY: The following code is both panic- and observation-safe, so it's ok to
            // create a temporary.
            let tmp = ptr::read(left_ptr);
            ptr::copy_nonoverlapping(right_ptr, left_ptr, 1);

            let mut i = 1;
            while i < lt_count && left_idx!(i) < lt_count {
                left_ptr = arr_ptr.add(left_idx!(i));
                ptr::copy_nonoverlapping(left_ptr, right_ptr, 1);
                right_ptr = arr_ptr.add(right_idx!(i));
                ptr::copy_nonoverlapping(right_ptr, left_ptr, 1);

                i += 1;
            }

            ptr::copy_nonoverlapping(&tmp, right_ptr, 1);
            mem::forget(tmp);
        }

        lt_count
    }
}

impl<T> SmallPartitionImpl for T {
    default fn partition<F>(v: &mut [Self], pivot: &Self, is_less: &mut F) -> usize
    where
        F: FnMut(&Self, &Self) -> bool,
    {
        small_partition_default(v, pivot, is_less)
    }
}

impl<T: Freeze + Copy> SmallPartitionImpl for T {
    fn partition<F>(v: &mut [Self], pivot: &Self, is_less: &mut F) -> usize
    where
        F: FnMut(&Self, &Self) -> bool,
    {
        if const { mem::size_of::<T>() <= mem::size_of::<[usize; 2]>() } {
            let len = v.len();

            if len > MAX_SMALL_PARTITION_LEN {
                debug_assert!(false);
                return 0;
            }

            let arr_ptr = v.as_mut_ptr();

            // SAFETY: TODO
            unsafe {
                let mut scratch = MaybeUninit::<[T; MAX_SMALL_PARTITION_LEN]>::uninit();
                let scratch_ptr = scratch.as_mut_ptr() as *mut T;

                let mut lt_count = 0;
                let mut ge_out_base_ptr = scratch_ptr.add(len);

                // LLVM unrolls this where appropriate in testing.
                for i in 0..len {
                    ge_out_base_ptr = ge_out_base_ptr.sub(1);
                    let elem_ptr = arr_ptr.add(i);

                    let is_lt = is_less(&*elem_ptr, pivot);

                    if const { mem::size_of::<T>() <= mem::size_of::<usize>() } {
                        ptr::copy_nonoverlapping(elem_ptr, scratch_ptr.add(lt_count), 1);
                        ptr::copy_nonoverlapping(elem_ptr, ge_out_base_ptr.add(lt_count), 1);
                    } else {
                        let dest_ptr = if is_lt { scratch_ptr } else { ge_out_base_ptr };
                        ptr::copy_nonoverlapping(elem_ptr, dest_ptr, 1);
                    }

                    lt_count += is_lt as usize;
                }

                // SAFETY: swap now contains all elements that belong on the left side of the pivot.
                // All comparisons have been done if is_less would have panicked `v` would have
                // stayed untouched.
                ptr::copy_nonoverlapping(scratch_ptr, arr_ptr, len);

                lt_count
            }
        } else {
            small_partition_default(v, pivot, is_less)
        }
    }
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // Optimized for efficiently handling small (sub-)slices while also being binary-size and
    // compile-time efficient.

    <T as SmallPartitionImpl>::partition(v, pivot, is_less)
}
