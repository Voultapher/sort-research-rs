use std::cmp::Ordering;

use rayon::slice::ParallelSliceMut;

sort_impl!("rust_rayon_parallel_stable");

trait RayonStableSort: Sized {
    fn sort(data: &mut [Self]);
}

impl<T> RayonStableSort for T {
    default fn sort(_data: &mut [Self]) {
        panic!("Type not supported.");
    }
}

impl<T: Send + Ord> RayonStableSort for T {
    fn sort(data: &mut [Self]) {
        data.par_sort_unstable();
    }
}

trait RayonStableSortBy<F>: Sized {
    fn sort_by(data: &mut [Self], compare: F);
}

impl<T, F> RayonStableSortBy<F> for T {
    default fn sort_by(_data: &mut [T], _compare: F) {
        panic!("Type not supported.");
    }
}

impl<T: Send + Ord, F: Fn(&T, &T) -> Ordering + Send + Sync> RayonStableSortBy<F> for T {
    fn sort_by(data: &mut [T], compare: F) {
        data.par_sort_unstable_by(compare);
    }
}

pub fn sort<T: Ord>(data: &mut [T]) {
    <T as RayonStableSort>::sort(data);
}

pub fn sort_by<T, F: FnMut(&T, &T) -> Ordering>(data: &mut [T], compare: F) {
    <T as RayonStableSortBy<F>>::sort_by(data, compare);
}
