partition_point_impl!("branchless_clean");

pub fn partition_point<T, P>(arr: &[T], mut pred: P) -> usize
where
    P: FnMut(&T) -> bool,
{
    // There are arr.len() + 1 possible outcomes of our search.
    // Invariant: [i+1, i+1+n) contains our desired result.
    let mut n = arr.len() + 1;
    let mut i = (-1isize) as usize;

    while n > 1 {
        // SAFETY
        //
        // First a lemma: n always ends up with the value 1.
        // This is true since n only ever gets decreased, by floor(n/2),
        // and thus there is no way to skip from a value >= 2 to 0.

        // mid >= 0 is trivial, as n > 1 implies n / 2 >= 1, and i is initially -1.
        // mid < arr.len() also holds, as i and mid are only ever increased by the
        // same amount as n is decreased. Since n ends with value 1 and starts with
        // arr.len() + 1, it is decreased by exactly arr.len(). Thus we find that
        // since i = -1 initially, that i and mid are at most -1 + arr.len().
        let mid = i.wrapping_add(n / 2);
        unsafe { core::intrinsics::assume(mid < arr.len()) }

        // We split our range [i+1, i+1+n) containing our result into two:
        // [i+1, i+1+n-floor(n/2)) and [i+1+floor(n/2), i+1+n)
        // Both ranges have length n - floor(n/2), which means together
        // they cover the complete original range. We test our predicate at
        // pred(a[i+floor(n/2)]), which if true means our result lies in
        // the latter range, and if false in the former (it might lie in both).

        // TODO explain black_box
        i = core::hint::black_box(if pred(&arr[mid]) { mid } else { i });
        n -= n / 2;
    }

    // [i+1, i+1+n) contains our result, and n == 1.
    i + 1
}
