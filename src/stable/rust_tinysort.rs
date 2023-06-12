use std::cmp::Ordering;

use tiny_sort;

sort_impl!("rust_tinymergesort_stable");

pub fn sort<T: Ord>(data: &mut [T]) {
    tiny_sort::stable::sort(data);
}

pub fn sort_by<T, F: FnMut(&T, &T) -> Ordering>(data: &mut [T], compare: F) {
    tiny_sort::stable::sort_by(data, compare);
}
