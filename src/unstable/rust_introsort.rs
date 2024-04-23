use std::cell::RefCell;
use std::cmp::Ordering;

sort_impl!("rust_introsort_unstable");

pub fn sort<T: Ord>(data: &mut [T]) {
    introsort::sort(data);
}

pub fn sort_by<T, F: FnMut(&T, &T) -> Ordering>(data: &mut [T], compare: F) {
    let compare_mut_cell = RefCell::new(compare);
    let compare_const = move |a, b| (*compare_mut_cell.borrow_mut())(a, b);

    introsort::sort_by(data, &compare_const);
}
