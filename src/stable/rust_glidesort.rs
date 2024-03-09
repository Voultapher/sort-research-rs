use std::cmp::Ordering;

sort_impl!("rust_glidesort_stable");

pub fn sort<T: Ord>(data: &mut [T]) {
    glidesort::sort(data);
}

pub fn sort_by<T, F: FnMut(&T, &T) -> Ordering>(data: &mut [T], compare: F) {
    glidesort::sort_by(data, compare);
}
