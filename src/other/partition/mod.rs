/// Various partition implementations.

pub trait Partition {
    fn name() -> String;

    fn partition<T>(arr: &mut [T], pivot: &T) -> usize
    where
        T: Ord;

    fn partition_by<T, F>(arr: &mut [T], pivot: &T, is_less: &mut F) -> usize
    where
        F: FnMut(&T, &T) -> bool;
}

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

pub mod block_quicksort;
pub mod crumsort;
pub mod ilp_partition;
pub mod new_block_quicksort;
pub mod simple_scan_branchless;
pub mod simple_scan_branchy;
pub mod sum_is_less;
