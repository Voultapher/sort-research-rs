//! Various partition implementations.

use std::alloc;
use std::cell::RefCell;
use std::ptr::NonNull;

use sort_test_tools::Partition;

macro_rules! partition_impl {
    ($name:expr) => {
        pub struct PartitionImpl;

        impl crate::other::partition::Partition for PartitionImpl {
            fn name() -> String {
                $name.into()
            }

            #[inline]
            fn partition<T>(arr: &mut [T], pivot: &T) -> usize
            where
                T: Ord,
            {
                partition(arr, pivot, &mut |a, b| a.lt(b))
            }

            #[inline]
            fn partition_by<T, F>(arr: &mut [T], pivot: &T, is_less: &mut F) -> usize
            where
                F: FnMut(&T, &T) -> bool,
            {
                partition(arr, pivot, is_less)
            }
        }
    };
}

/// Returns a guaranteed non-null pointer to an allocation suitable for `layout`.
///
/// As long as this function is called consecutively with the same `layout`, it will re-use the same
/// allocation. This makes this function suitable in a benchmark scenario where a function is tested
/// repeatedly requesting the same layout.
///
/// If the allocation fails, a panic is raised.
pub fn get_or_alloc_tls_scratch(layout: alloc::Layout) -> NonNull<u8> {
    struct ScratchCache {
        layout: alloc::Layout,
        ptr: NonNull<u8>,
    }

    impl ScratchCache {
        fn new(layout: alloc::Layout) -> Self {
            Self {
                layout,
                ptr: NonNull::new(unsafe { alloc::alloc(layout) }).unwrap(),
            }
        }

        fn get_or_replace(&mut self, layout: alloc::Layout) -> NonNull<u8> {
            if self.layout.align() != layout.align() || self.layout.size() < layout.size() {
                *self = Self::new(layout);
            }

            self.ptr
        }
    }

    impl Drop for ScratchCache {
        fn drop(&mut self) {
            unsafe { alloc::dealloc(self.ptr.as_ptr(), self.layout) }
        }
    }

    thread_local! {
        static SCRATCH_CACHE: RefCell<Option<ScratchCache>> = RefCell::new(None);
    }

    SCRATCH_CACHE.with(|scratch_cache_opt| {
        scratch_cache_opt
            .borrow_mut()
            .get_or_insert_with(|| ScratchCache::new(layout))
            .get_or_replace(layout)
    })
}

pub mod hoare_block;
pub mod hoare_block_butterfly;
pub mod hoare_branchy;
pub mod hoare_branchy_cyclic;
pub mod hoare_crumsort;
pub mod hoare_crumsort_rs;
pub mod hybrid_bitset_partition;
pub mod hybrid_block_partition;
pub mod lomuto_branchless;
pub mod lomuto_branchless_cyclic;
pub mod lomuto_branchless_cyclic_opt;
pub mod lomuto_branchy;
pub mod lomuto_iterleaved;
pub mod lomuto_nanosort;
pub mod small_partition;
pub mod stable_2side_fill;
pub mod sum_is_less;
