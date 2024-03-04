use std::cmp::Ordering;

sort_impl!("rust_ipnsort_unstable");

pub fn sort<T: Ord>(data: &mut [T]) {
    ipnsort::sort(data);
}

pub fn sort_by<T, F: FnMut(&T, &T) -> Ordering>(data: &mut [T], compare: F) {
    ipnsort::sort_by(data, compare);
}
