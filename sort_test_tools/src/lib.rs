#![feature(vec_into_raw_parts)]

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
