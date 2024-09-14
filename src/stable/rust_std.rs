use std::cmp::Ordering;

sort_impl!("rust_std_stable");

#[inline]
pub fn sort<T>(v: &mut [T])
where
    T: Ord,
{
    v.sort();
}

#[inline]
pub fn sort_by<T, F>(v: &mut [T], compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    v.sort_by(compare);
}
