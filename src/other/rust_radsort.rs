use std::cmp::Ordering;

sort_impl!("rust_radsort_radix");

trait RadSort: Sized {
    fn sort(data: &mut [Self]);
}

impl<T> RadSort for T {
    default fn sort(_data: &mut [Self]) {
        panic!("Type not supported by radsort");
    }
}

impl<T: radsort::Key> RadSort for T {
    fn sort(data: &mut [Self]) {
        radsort::sort(data);
    }
}

pub fn sort<T: Ord>(data: &mut [T]) {
    RadSort::sort(data);
}

pub fn sort_by<T, F: FnMut(&T, &T) -> Ordering>(_data: &mut [T], _compare: F) {
    panic!("sort_by not supported by radsort");
}
