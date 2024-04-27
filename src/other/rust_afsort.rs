use std::cmp::Ordering;

use afsort::AFSortable;

sort_impl!("rust_afsort_radix");

trait AFSort: Sized {
    fn sort(data: &mut [Self]);
}

impl<T> AFSort for T {
    default fn sort(_data: &mut [Self]) {
        panic!("Type not supported");
    }
}

impl<T> AFSort for T
where
    [T]: AFSortable,
{
    fn sort(data: &mut [Self]) {
        data.af_sort_unstable();
    }
}

pub fn sort<T: Ord>(data: &mut [T]) {
    AFSort::sort(data);
}

pub fn sort_by<T, F: FnMut(&T, &T) -> Ordering>(_data: &mut [T], _compare: F) {
    panic!("sort_by not supported");
}
