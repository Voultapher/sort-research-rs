use std::cmp::Ordering;

sort_impl!("rust_tinyheapsort_unstable");

pub fn sort<T: Ord>(data: &mut [T]) {
    tiny_sort::unstable::sort(data);
}

pub fn sort_by<T, F: FnMut(&T, &T) -> Ordering>(data: &mut [T], compare: F) {
    tiny_sort::unstable::sort_by(data, compare);
}
