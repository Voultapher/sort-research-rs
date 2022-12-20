partition_impl!("sum_is_less");

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let check_idx = v
        .iter()
        .map(|elem| is_less(elem, pivot) as usize)
        .sum::<usize>();

    check_idx
}
