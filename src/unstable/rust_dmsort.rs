use std::cmp::Ordering;

sort_impl!("rust_dmsort_unstable");

pub fn sort<T: Ord>(data: &mut [T]) {
    dmsort::sort(data);
}

pub fn sort_by<T, F: FnMut(&T, &T) -> Ordering>(data: &mut [T], compare: F) {
    dmsort::sort_by(data, compare);
}
