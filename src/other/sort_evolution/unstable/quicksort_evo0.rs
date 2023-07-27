//! Basic recursive quicksort.

use std::cmp::Ordering;
use std::mem::SizedTypeProperties;

sort_impl!("quicksort_evo0_unstable");

#[inline]
pub fn sort<T>(v: &mut [T])
where
    T: Ord,
{
    unstable_sort(v, |a, b| a.lt(b));
}

#[inline]
pub fn sort_by<T, F>(v: &mut [T], mut compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    unstable_sort(v, |a, b| compare(a, b) == Ordering::Less);
}

////////////////////////////////////////////////////////////////////////////////
// Sorting
////////////////////////////////////////////////////////////////////////////////

#[inline]
#[cfg(not(no_global_oom_handling))]
fn unstable_sort<T, F>(v: &mut [T], mut is_less: F)
where
    F: FnMut(&T, &T) -> bool,
{
    if T::IS_ZST {
        // Sorting has no meaningful behavior on zero-sized types. Do nothing.
        return;
    }

    quicksort(v, &mut is_less);
}

fn quicksort<T, F>(mut v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    loop {
        let len = v.len();
        if len < 2 {
            return;
        }

        // Leverage ASLR for pseudo-random pivot selection.
        // let pivot_pos = 0; //(v.as_ptr() as usize) % len;
        let (pivot, v_without_pivot) = v.split_at_mut(1);
        let pivot = &pivot[0];

        let lt_count = lomuto_partition(v_without_pivot, pivot, is_less);

        // Place the pivot between the two partitions.
        v.swap(0, lt_count);

        // Recurse into the left side.
        quicksort(&mut v[..lt_count], is_less);

        // Continue with the right side.
        v = &mut v[(lt_count + 1)..];
    }
}

fn lomuto_partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    let mut l = 0;
    for r in 0..len {
        if is_less(&v[r], pivot) {
            v.swap(l, r);
            l += 1;
        }
    }

    l
}
