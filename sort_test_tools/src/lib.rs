#![feature(vec_into_raw_parts, macro_metavar_expr_concat)]

pub trait Sort {
    fn name() -> String;

    fn sort<T>(v: &mut [T])
    where
        T: Ord;

    fn sort_by<T, F>(v: &mut [T], compare: F)
    where
        F: FnMut(&T, &T) -> std::cmp::Ordering;
}

pub mod ffi_types;
pub mod patterns;
pub mod tests;
