use core::cmp::Ordering;

sort_impl!("sort4_unstable_branchy");

#[inline(never)]
fn sort_network_4<T, F>(arr: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    assert!(arr.len() == 4);

    swap(arr, 0, 1, is_less);
    swap(arr, 2, 3, is_less);
    swap(arr, 0, 2, is_less);
    swap(arr, 1, 3, is_less);
    swap(arr, 1, 2, is_less);
}

fn swap<T, F>(arr: &mut [T], i: usize, j: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    if is_less(&arr[j], &arr[i]) {
        arr.swap(i, j);
    }
}

fn sort_impl<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    sort_network_4(v, is_less);
}

fn sort<T: Ord>(v: &mut [T]) {
    sort_impl(v, &mut |a, b| a.lt(b));
}

fn sort_by<T, F>(v: &mut [T], mut compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    sort_impl(v, &mut |a, b| compare(a, b) == Ordering::Less);
}
