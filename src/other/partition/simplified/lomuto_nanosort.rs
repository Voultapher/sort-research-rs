//! Inspired by https://github.com/zeux/nanosort

fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let mut left = 0;

    for right in 0..v.len() {
        // SAFETY: `left` can at max be incremented by 1 each loop iteration, which implies:
        // unsafe { core::intrinsics::assume(left <= right) };

        let right_is_lt = is_less(&v[right], pivot);
        v.swap(left, right);
        left += right_is_lt as usize;
    }

    left
}
