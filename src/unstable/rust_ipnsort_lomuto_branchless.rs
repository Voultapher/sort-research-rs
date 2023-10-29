use std::cmp::Ordering;

use ipnsort;

use crate::other::partition;

sort_impl!("lomuto_branchless");

pub fn sort<T: Ord>(data: &mut [T]) {
    ipnsort::sort(data, partition::lomuto_branchless::PartitionImpl);
}

pub fn sort_by<T, F: FnMut(&T, &T) -> Ordering>(data: &mut [T], compare: F) {
    ipnsort::sort_by(data, compare, partition::lomuto_branchless::PartitionImpl);
}
