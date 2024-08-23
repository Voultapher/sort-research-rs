#![feature(vec_into_raw_parts, macro_metavar_expr_concat)]

pub trait Sort {
    fn name() -> String;

    fn sort<T>(arr: &mut [T])
    where
        T: Ord;

    fn sort_by<T, F>(arr: &mut [T], compare: F)
    where
        F: FnMut(&T, &T) -> std::cmp::Ordering;
}

pub mod ffi_types;
pub mod patterns;
pub mod tests;

mod known_good_stable_sort;
mod zipf;
