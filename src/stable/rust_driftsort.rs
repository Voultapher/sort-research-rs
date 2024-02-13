use std::cmp::Ordering;

use driftsort;

sort_impl!("rust_driftsort_stable");

pub fn sort<T: Ord>(data: &mut [T]) {
    driftsort::sort(data);
}

pub fn sort_by<T, F: FnMut(&T, &T) -> Ordering>(data: &mut [T], compare: F) {
    driftsort::sort_by(data, compare);
}
