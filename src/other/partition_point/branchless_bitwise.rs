partition_point_impl!("branchless_bitwise");

pub fn partition_point<T, P>(arr: &[T], mut pred: P) -> usize
where
    P: FnMut(&T) -> bool,
{
    let len = arr.len();
    if len == 0 {
        return 0;
    }

    let mut k = len.ilog2() as usize;
    let mut b = if pred(&arr[len / 2]) {
        len.wrapping_sub(1 << k)
    } else {
        usize::MAX
    };

    while k != 0 {
        k -= 1;
        // SAFETY: TODO
        let should_add = unsafe { pred(arr.get_unchecked(b.wrapping_add(1 << k))) };
        b = b.wrapping_add((should_add as usize) << k);
    }

    b + 1
}
