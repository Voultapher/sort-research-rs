partition_point_impl!("std");

pub fn partition_point<T, P>(arr: &[T], mut pred: P) -> usize
where
    P: FnMut(&T) -> bool,
{
    // std impl as of Rust 1.69

    use std::cmp::Ordering::{Greater, Less};

    arr.binary_search_by(|x| if pred(x) { Less } else { Greater })
        .unwrap_or_else(|i| i)
}
