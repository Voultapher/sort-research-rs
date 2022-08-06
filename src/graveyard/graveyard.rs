// Sorting functions and blocks that were discarded during development for one reason or another.

fn sort3<T, F>(x1: &mut T, x2: &mut T, x3: &mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // stable, 2-3 compares, 0-2 swaps

    if !is_less(x2, x1) {
        if !is_less(x3, x2) {
            return;
        }

        mem::swap(x2, x3);
        if is_less(x2, x1) {
            mem::swap(x1, x2);
        }
        return;
    }

    if is_less(x3, x2) {
        mem::swap(x1, x3);
        return;
    }

    mem::swap(x1, x2);
    if is_less(x3, x2) {
        mem::swap(x2, x3);
    }
}

fn sort4<T, F>(x1: &mut T, x2: &mut T, x3: &mut T, x4: &mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // stable, 3-6 compares, 0-5 swaps

    sort3(x1, x2, x3, is_less);

    if is_less(x4, x3) {
        mem::swap(x3, x4);
        if is_less(x3, x2) {
            mem::swap(x2, x3);
            if is_less(x2, x1) {
                mem::swap(x1, x2);
            }
        }
    }
}

fn sort5<T, F>(x1: &mut T, x2: &mut T, x3: &mut T, x4: &mut T, x5: &mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // stable, 4-10 compares, 0-9 swaps

    sort4(x1, x2, x3, x4, is_less);

    if is_less(x5, x4) {
        mem::swap(x4, x5);
        if is_less(x4, x3) {
            mem::swap(x3, x4);
            if is_less(x3, x2) {
                mem::swap(x2, x3);
                if is_less(x2, x1) {
                    mem::swap(x1, x2);
                }
            }
        }
    }
}

unsafe fn quad_swap_four<T, F>(arr: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    debug_assert!(arr.len() >= 4);

    let mut arr_ptr = arr.as_mut_ptr();

    // Important to only swap if it is more and not if it is equal.
    // is_less should return false for equal, so we don't swap.
    swap_next_if(arr_ptr, !is_less(&*arr_ptr, &*arr_ptr.add(1))); // arr[0/1]

    arr_ptr = arr_ptr.add(2); // Total offset
    swap_next_if(arr_ptr, !is_less(&*arr_ptr, &*arr_ptr.add(1))); // arr[2/3]

    arr_ptr = arr_ptr.offset(-1); // Total offset 1

    if !is_less(&*arr_ptr, &*arr_ptr.add(1)) {
        arr_ptr.swap(arr_ptr.add(1)); // arr[1/2]

        arr_ptr = arr_ptr.offset(-1); // Total offset 0
        swap_next_if(arr_ptr, !is_less(&*arr_ptr, &*arr_ptr.add(1))); // arr[0/1]

        arr_ptr = arr_ptr.add(2); // Total offset 2
        swap_next_if(arr_ptr, !is_less(&*arr_ptr, &*arr_ptr.add(1))); // arr[2/3]

        arr_ptr = arr_ptr.offset(-1); // Total offset 1
        swap_next_if(arr_ptr, !is_less(&*arr_ptr, &*arr_ptr.add(1))); // arr[1/2]
    }
}

// BUGGY not stable.
/// Sort the remaining elements after offset in arr.
unsafe fn sort_insertion_remaining<T, F>(arr: &mut [T], offset: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // Safety: offset must be larger than 1.
    debug_assert!(offset >= 4);

    let arr_ptr = arr.as_mut_ptr();

    for i in offset..arr.len() {
        let mut end = arr_ptr.add(i);
        let mut pta = end.offset(-1);

        if is_less(&*pta, &*end) {
            continue;
        }

        // Give ourselves some scratch space to work with.
        // We do not have to worry about drops: `MaybeUninit` does nothing when dropped.
        let mut tmp = mem::MaybeUninit::<T>::uninit();
        ptr::copy_nonoverlapping(end, tmp.as_mut_ptr(), 1);

        if is_less(&*arr_ptr, &*end) {
            // Rotate left until TODO?
            loop {
                ptr::copy_nonoverlapping(pta, end, 1);
                pta = pta.offset(-1);
                end = end.offset(-1);

                // The original did two shifts here, but I'm pretty sure that's a bug and UB.

                if is_less(&*pta, &*tmp.as_ptr()) {
                    break;
                }
            }

            ptr::copy_nonoverlapping(end.add(1), end, 1);
            ptr::copy_nonoverlapping(tmp.as_ptr(), end, 1);
        } else {
            // Rotate left until TODO?
            let mut top = i - 1;

            while top > 0 {
                ptr::copy_nonoverlapping(pta, end, 1);
                pta = pta.offset(-1);
                end = end.offset(-1);

                top -= 1;
            }

            ptr::copy_nonoverlapping(tmp.as_ptr(), end, 1);
            end = end.offset(-1);
        }

        swap_next_if_less(end, is_less);
    }
}

/// Sort the remaining elements after offset in arr.
fn sort_insertion_remaining<T, F>(arr: &mut [T], offset: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let arr_ptr = arr.as_mut_ptr();

    let len = arr.len();

    debug_assert!(offset >= 1);
    let mut i = offset;

    unsafe {
        // Port of https://en.wikipedia.org/wiki/Insertion_sort.
        while i < len {
            // Give ourselves some scratch space to work with.
            // We do not have to worry about drops: `MaybeUninit` does nothing when dropped.
            let mut x = mem::MaybeUninit::<T>::uninit();
            ptr::copy_nonoverlapping(arr_ptr.add(i), x.as_mut_ptr(), 1);

            let mut j = (i as isize) - 1;
            while j >= 0 && is_less(&*x.as_ptr(), arr.get_unchecked(j as usize)) {
                ptr::copy_nonoverlapping(arr_ptr.add(j as usize), arr_ptr.add((j + 1) as usize), 1);

                // TODO can this be done with less branches?
                j -= 1;
            }

            ptr::copy_nonoverlapping(x.as_ptr(), arr_ptr.add((j + 1) as usize), 1);
            i += 1;
        }
    }
}

// Super simple insertion sort.
/// Sort the remaining elements after offset in arr.
unsafe fn sort_insertion_remaining<T, F>(arr: &mut [T], offset: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let arr_ptr = arr.as_mut_ptr();
    let len = arr.len();

    let mut i = offset;

    while i < len {
        let mut j = i;

        loop {
            if j == 0 {
                break;
            }

            let x1 = arr_ptr.add(j - 1);
            let x2 = arr_ptr.add(j);

            if !is_less(&*x2, &*x1) {
                break;
            }

            ptr::swap_nonoverlapping(x1, x2, 1);
            j -= 1;
        }

        i += 1;
    }
}

// slower than sort4 + parity merge

mod parity_merge_impl {
    use super::*;

    #[inline]
    pub unsafe fn merge_up<T, F>(
        mut ptr_left: *mut T,
        mut ptr_right: *mut T,
        mut ptr_swap: *mut T,
        is_less: &mut F,
    ) -> (*mut T, *mut T, *mut T)
    where
        F: FnMut(&T, &T) -> bool,
    {
        // C: x = cmp(ptl, ptr) <= 0; y = !x; pts[x] = *ptr; ptr += y; pts[y] = *ptl; ptl += x; pts++;
        let x = !is_less(&*ptr_right, &*ptr_left);
        let y = !x;
        ptr::copy_nonoverlapping(ptr_right, ptr_swap.add(x as usize), 1);
        ptr_right = ptr_right.add(y as usize);
        ptr::copy_nonoverlapping(ptr_left, ptr_swap.add(y as usize), 1);
        ptr_left = ptr_left.add(x as usize);
        ptr_swap = ptr_swap.add(1);

        (ptr_left, ptr_right, ptr_swap)
    }

    #[inline]
    pub unsafe fn merge_down<T, F>(
        mut ptr_left: *mut T,
        mut ptr_right: *mut T,
        mut ptr_swap: *mut T,
        is_less: &mut F,
    ) -> (*mut T, *mut T, *mut T)
    where
        F: FnMut(&T, &T) -> bool,
    {
        // C: x = cmp(ptl, ptr) <= 0; y = !x; pts--; pts[x] = *ptr; ptr -= x; pts[y] = *ptl; ptl -= y;
        let x = !is_less(&*ptr_right, &*ptr_left);
        let y = !x;
        ptr_swap = ptr_swap.offset(-1);
        ptr::copy_nonoverlapping(ptr_right, ptr_swap.add(x as usize), 1);
        ptr_right = ptr_right.offset(-(x as isize));
        ptr::copy_nonoverlapping(ptr_left, ptr_swap.add(y as usize), 1);
        ptr_left = ptr_left.offset(-(y as isize));

        (ptr_left, ptr_right, ptr_swap)
    }

    #[inline]
    pub unsafe fn finish_up<T, F>(
        ptr_left: *mut T,
        ptr_right: *mut T,
        ptr_swap: *mut T,
        is_less: &mut F,
    ) where
        F: FnMut(&T, &T) -> bool,
    {
        // C: *pts = cmp(ptl, ptr) <= 0 ? *ptl : *ptr;
        let copy_ptr = if is_less(&*ptr_right, &*ptr_left) {
            ptr_right
        } else {
            ptr_left
        };
        ptr::copy_nonoverlapping(copy_ptr, ptr_swap, 1);
    }

    #[inline]
    pub unsafe fn finish_down<T, F>(
        ptr_left: *mut T,
        ptr_right: *mut T,
        ptr_swap: *mut T,
        is_less: &mut F,
    ) where
        F: FnMut(&T, &T) -> bool,
    {
        // C: *pts = cmp(ptl, ptr)  > 0 ? *ptl : *ptr;
        let copy_ptr = if is_less(&*ptr_right, &*ptr_left) {
            ptr_left
        } else {
            ptr_right
        };
        ptr::copy_nonoverlapping(copy_ptr, ptr_swap, 1);
    }
} // mod parity_merge_impl

unsafe fn parity_merge2<T, F>(arr_ptr: *mut T, swap_ptr: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: TODO

    let mut ptr_left = arr_ptr;
    let mut ptr_right = arr_ptr.add(2);
    let mut ptr_swap = swap_ptr;

    (ptr_left, ptr_right, ptr_swap) =
        parity_merge_impl::merge_up(ptr_left, ptr_right, ptr_swap, is_less);

    parity_merge_impl::finish_up(ptr_left, ptr_right, ptr_swap, is_less);

    // ---

    ptr_left = arr_ptr.add(1);
    ptr_right = arr_ptr.add(3);
    ptr_swap = swap_ptr.add(3);

    (ptr_left, ptr_right, ptr_swap) =
        parity_merge_impl::merge_down(ptr_left, ptr_right, ptr_swap, is_less);

    parity_merge_impl::finish_down(ptr_left, ptr_right, ptr_swap, is_less);
}

unsafe fn parity_merge4<T, F>(arr_ptr: *mut T, swap_ptr: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: TODO

    let mut ptr_left = swap_ptr;
    let mut ptr_right = swap_ptr.add(4);
    let mut ptr_swap = arr_ptr;

    (ptr_left, ptr_right, ptr_swap) =
        parity_merge_impl::merge_up(ptr_left, ptr_right, ptr_swap, is_less);
    (ptr_left, ptr_right, ptr_swap) =
        parity_merge_impl::merge_up(ptr_left, ptr_right, ptr_swap, is_less);
    (ptr_left, ptr_right, ptr_swap) =
        parity_merge_impl::merge_up(ptr_left, ptr_right, ptr_swap, is_less);

    parity_merge_impl::finish_up(ptr_left, ptr_right, ptr_swap, is_less);

    // ---

    ptr_left = swap_ptr.add(3);
    ptr_right = swap_ptr.add(7);
    ptr_swap = arr_ptr.add(7);

    (ptr_left, ptr_right, ptr_swap) =
        parity_merge_impl::merge_down(ptr_left, ptr_right, ptr_swap, is_less);
    (ptr_left, ptr_right, ptr_swap) =
        parity_merge_impl::merge_down(ptr_left, ptr_right, ptr_swap, is_less);
    (ptr_left, ptr_right, ptr_swap) =
        parity_merge_impl::merge_down(ptr_left, ptr_right, ptr_swap, is_less);

    parity_merge_impl::finish_down(ptr_left, ptr_right, ptr_swap, is_less);
}

/// Sort the first 8 elements of arr.
unsafe fn sort8<T, F>(arr: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure arr is at least len 8.
    debug_assert!(arr.len() >= 8);

    let mut arr_ptr = arr.as_mut_ptr();
    swap_next_if_less(arr_ptr, is_less);

    arr_ptr = arr_ptr.add(2);
    swap_next_if_less(arr_ptr, is_less);

    arr_ptr = arr_ptr.add(2);
    swap_next_if_less(arr_ptr, is_less);

    arr_ptr = arr_ptr.add(2);
    swap_next_if_less(arr_ptr, is_less);

    arr_ptr = arr.as_mut_ptr();
    if !is_less(&*arr_ptr.add(2), &*arr_ptr.add(1))
        && !is_less(&*arr_ptr.add(4), &*arr_ptr.add(3))
        && !is_less(&*arr_ptr.add(6), &*arr_ptr.add(5))
    {
        return;
    }

    // TODO alloc for larger types.
    let mut swap = mem::MaybeUninit::<[T; 8]>::uninit();
    let swap_ptr = swap.as_mut_ptr() as *mut T;

    parity_merge2(arr_ptr, swap_ptr, is_less);
    parity_merge2(arr_ptr.add(4), swap_ptr.add(4), is_less);

    parity_merge4(arr_ptr, swap_ptr, is_less);
}

// --- no used again inline directly ---

#[inline]
fn tail_swap<T, F>(arr: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = arr.len();

    // This function should not be called for larger slices.
    debug_assert!(len < 32);

    // if (nmemb < 4) {
    //     FUNC(bubble_sort)(array, nmemb, cmp);
    //     return;
    // }
    // if (nmemb < 8) {
    //     FUNC(quad_swap_four)(array, cmp);
    //     FUNC(sort_insertion_remaining)(array, 4, nmemb, cmp);
    //     return;
    // }
    // if (nmemb < 16) {
    //     FUNC(parity_swap_eight)(array, cmp);
    //     FUNC(sort_insertion_remaining)(array, 8, nmemb, cmp);
    //     return;
    // }
    // FUNC(parity_swap_sixteen)(array, cmp);
    // FUNC(sort_insertion_remaining)(array, 16, nmemb, cmp);
}

// --- buggy

#[inline]
fn flux_analyze<T, F>(arr: &[T], is_less: &mut F) -> SortStrategy
where
    F: FnMut(&T, &T) -> bool,
{
    const STREAK_SIZE: usize = 16;

    // The logic of this function is not built for smaller slices.
    let len = arr.len();
    debug_assert!(len > STREAK_SIZE);
    // The inner chunking loop is wrong if these are not given.
    debug_assert!(STREAK_SIZE >= 2 && STREAK_SIZE % 2 == 0);

    let mut balance = 0usize;
    let mut streaks = 0usize;

    if len == 24 {
        let x = 3;
    }

    let chunk_end = (arr.len() / STREAK_SIZE) * STREAK_SIZE;

    let mut i = 0;
    while i < chunk_end {
        let mut dist = 0;

        for j in i..(i + STREAK_SIZE) {
            // SAFETY: We know that chunk is at least 2 and even sized. So that all window accesses
            // are safe to do.
            // PANIC SAFETY: we only have read access to arr.
            unsafe {
                dist += is_less(arr.get_unchecked(j + 1), arr.get_unchecked(j)) as usize;
            }
        }

        // Streak means all ascending or descending.
        streaks += ((dist == 0) | (dist == STREAK_SIZE)) as usize;
        balance += dist;
        i += STREAK_SIZE;
    }

    for i in chunk_end.saturating_sub(1)..len.saturating_sub(1) {
        // SAFETY: We start at zero >= i < len and read until arr[len - 1 - 1 + 1].
        //
        // PANIC SAFETY: we only have read access to arr.
        unsafe {
            balance += is_less(arr.get_unchecked(i + 1), arr.get_unchecked(i)) as usize;
        }
    }

    match balance {
        0 => SortStrategy::AlreadySorted,
        _ if balance == len - 1 => SortStrategy::Reverse,
        _ if streaks >= len / 40 => SortStrategy::Merge,
        _ => SortStrategy::Quick,
    }
}

// for window in arr.windows(2) {
//     // SAFETY: We know window is of size 2.
//     let a = unsafe { window.get_unchecked(0) };
//     let b = unsafe { window.get_unchecked(1) };

//     let is_pair_less = is_less(a, b);

//     // TODO explain branchless bit shifting.
//     let streak_bit = (is_pair_less as u8).wrapping_shl(i);
//     streak_up |= streak_bit;
//     let is_new_streak_up = streak_up == u8::MAX;
//     streak_up *= (!is_new_streak_up) as u8;

//     streak_down ^= (is_pair_less as u8 ^ streak_down) & (1u8.wrapping_shl(i));
//     // streak_down ^= streak_bit;

//     println!(
//         "streak_up: {:08b} streak_down: {:08b} is_new_streak_up: {is_new_streak_up},",
//         streak_up, streak_down,
//     );

//     // streak_window |=
//     // let is_new_streak = (streak_window == 0) | (streak_window == u8::MAX);
//     // let bitmask = (is_new_streak as u8 * u8::MAX);
//     // streak_window = streak_window ^ bitmask;

//     sorted += is_pair_less as usize;
//     // streaks += is_new_streak as usize;

//     i = i.wrapping_add(1);
// }

// -- This was painful to throw away :(

#[inline] // FIXME should not be pub
pub fn flux_analyze<T, F>(arr: &[T], mut is_less: F) -> SortStrategy
where
    F: FnMut(&T, &T) -> bool,
{
    // IMPORTANT keep these in sync. There has to be a bit per streak entry.
    let mut streak_up: u16 = 0;
    let mut streak_down: u16 = 0;

    let mut sorted = 0usize;
    let mut streaks = 0usize;

    // TODO check how efficient the code gen is. https://godbolt.org/z/bo67W44WP not so great, will
    // try handwritten.
    let mut i = 0u32;

    for window in arr.windows(2) {
        // SAFETY: We know window is of size 2.
        let a = unsafe { window.get_unchecked(0) };
        let b = unsafe { window.get_unchecked(1) };

        let is_pair_less = is_less(a, b);

        // Analyze slice to find out if there are STREAK_SIZE consecutive elements that are all
        // either ascending or descending. This implementation is branchless and doesn't depend on a
        // fixed chunk size and boundary.

        // Setup streak bit, either all zeros or a one at a shifting location each iteration.
        // Eg. 00010000, next iteration 00100000
        let streak_bit_up = (is_pair_less as u16).wrapping_shl(i);
        let streak_bit_down = (!is_pair_less as u16).wrapping_shl(i);

        // Set streak_up bit eg. 00010000 -> 00110000
        streak_up |= streak_bit_up;
        streak_down |= streak_bit_down;

        // Once there have STREAK_SIZE consecutive elements that were either all less or more then,
        // is_streak_up becomes true.
        let is_streak = streak_up == u16::MAX || streak_down == u16::MAX;

        // If is_streak == true reset both streak trackers to 0.
        streak_up *= !is_streak as u16;
        streak_down *= !is_streak as u16;

        sorted += is_pair_less as usize;
        streaks += is_streak as usize;

        i = i.wrapping_add(1);
    }

    let len = arr.len();
    match sorted {
        0 => SortStrategy::Reverse,
        _ if sorted == len - 1 => SortStrategy::AlreadySorted,
        _ if streaks >= len / 32 => SortStrategy::Merge,
        _ => SortStrategy::Quick,
    }
}

// --- Back to single loop

pub fn flux_analyze<T, F>(arr: &[T], mut is_less: F) -> SortStrategy
where
    F: FnMut(&T, &T) -> bool,
{
    const STREAK_SIZE: usize = 16;

    // The logic of this function makes no sense for smaller slices.
    let len = arr.len();
    debug_assert!(len > STREAK_SIZE);

    let mut sorted = 0usize;
    let mut streaks = 0usize;

    let chunk_end = (len / STREAK_SIZE) * STREAK_SIZE;
    for i in 0..chunk_end.saturating_sub(1) {
        let mut batch_sorted = 0;

        for j in i..(i + STREAK_SIZE) {
            let a = unsafe { arr.get_unchecked(j) };
            let b = unsafe { arr.get_unchecked(j + 1) };

            batch_sorted = is_less(a, b) as usize;
        }

        sorted += batch_sorted;
        streaks += (batch_sorted == 0 || batch_sorted == STREAK_SIZE) as usize;
    }

    for i in chunk_end..len.saturating_sub(1) {
        let a = unsafe { arr.get_unchecked(i) };
        let b = unsafe { arr.get_unchecked(i + 1) };

        sorted = is_less(a, b) as usize;
    }

    let len = arr.len();
    match sorted {
        0 => SortStrategy::Reverse,
        _ if sorted == len - 1 => SortStrategy::AlreadySorted,
        _ if streaks >= len / 32 => SortStrategy::Merge,
        _ => SortStrategy::Quick,
    }
}

// --- I'm surprised how slow this is

fn flux_analyze<T, F>(arr: &[T], mut is_less: F) -> SortStrategy
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: If you set this to a very low value, the function might trigger UB.
    const STREAK_SIZE: usize = 16;

    let len = arr.len();
    if len <= STREAK_SIZE {
        // The logic of this function makes no sense for smaller slices.
        // The safety of an access later on depends on this property, so hard error.
        panic!("len too small for flux_analyze");
    }

    let mut sorted = 0usize;
    let mut streaks = 0usize;

    let mut chunk_i = 0usize;
    let mut dist = 0usize;
    let wrap_offset = len % STREAK_SIZE;
    let chunk_end = len.wrapping_sub(wrap_offset + 1);

    // We checked for len above. And - 1 because we look at a window.
    // Eg. len == 32 -> wrap_offset == 0 and we want to look at i + 1. So i may only be 30.
    for i in 0..chunk_end {
        let a = unsafe { arr.get_unchecked(i) };
        let b = unsafe { arr.get_unchecked(i + 1) };

        dist += is_less(a, b) as usize;
        chunk_i += 1;

        if chunk_i == (STREAK_SIZE - 1) {
            sorted += dist;
            streaks += (dist == (STREAK_SIZE - 1) || dist == 0) as usize;

            chunk_i = 0;
            dist = 0;
        }
    }

    sorted += dist;

    // -1 because we look at window.
    for i in chunk_end..len.wrapping_sub(1) {
        let a = unsafe { arr.get_unchecked(i) };
        let b = unsafe { arr.get_unchecked(i + 1) };

        sorted += is_less(a, b) as usize;
    }

    // match sorted {
    //     0 => SortStrategy::Reverse,
    //     _ if sorted == len - 1 => SortStrategy::AlreadySorted,
    //     _ if streaks >= len / 40 => SortStrategy::Merge,
    //     _ => SortStrategy::Quick,
    // }

    match sorted {
        0 => panic!("reverse"),
        _ if sorted == len - 1 => SortStrategy::AlreadySorted,
        _ if streaks >= len / 40 => panic!("merge sort"),
        _ => panic!("quick sort"),
    }
}

// {
//     // This is kinda unholy but tweaking the above loop is terrible for performance.
//     // TODO what if the list doesn't fit into cache? Is this worse?
//     let mut i = 0;
//     while i < chunk_end {
//         let a = unsafe { arr.get_unchecked(i) };
//         let b = unsafe { arr.get_unchecked(i + 1) };

//         sorted += is_less(a, b) as usize;
//         i += STREAK_SIZE;
//     }
// }
