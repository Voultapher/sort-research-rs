pub trait PartitionPoint {
    fn name() -> String;

    fn partition_point<T>(arr: &[T], val: &T) -> usize
    where
        T: Ord;

    fn partition_point_by<T, F>(arr: &[T], val: &T, compare: F) -> usize
    where
        F: FnMut(&T, &T) -> core::cmp::Ordering;
}

macro_rules! partition_point_impl {
    ($name:expr) => {
        pub struct PartitionPointImpl;

        impl crate::other::partition_point::PartitionPoint for PartitionPointImpl {
            fn name() -> String {
                $name.into()
            }

            #[inline]
            fn partition_point<T>(arr: &[T], val: &T) -> usize
            where
                T: Ord,
            {
                partition_point(arr, |elem| elem < val)
            }

            #[inline]
            fn partition_point_by<T, F>(arr: &[T], val: &T, mut compare: F) -> usize
            where
                F: FnMut(&T, &T) -> std::cmp::Ordering,
            {
                partition_point(arr, |elem| compare(elem, val).is_lt())
            }
        }
    };
}

pub mod branchless_bitwise;
pub mod branchless_clean;
pub mod std;
