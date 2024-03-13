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

// Wrote a faster less swap heavy version inspired by insert_head.

/// Sort the remaining elements after offset in v.
unsafe fn insertion_sort_remaining<T, F>(v: &mut [T], offset: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let arr_ptr = v.as_mut_ptr();
    let len = v.len();

    let mut i = offset;

    while i < len {
        let mut j = i;

        loop {
            if j == 0 {
                break;
            }

            let x1 = arr_ptr.add(j - 1);
            let x2 = arr_ptr.add(j);

            // PANIC SAFETY: if is_less panics, no scratch memory was created and the slice should
            // still be in a well defined state, without duplicates.
            if !is_less(&*x2, &*x1) {
                break;
            }

            ptr::swap_nonoverlapping(x1, x2, 1);
            j -= 1;
        }

        i += 1;
    }
}

// It's pretty but I'm not sure it is faster.
fn collapse(runs: &[Run]) -> Option<usize> {
    let n = runs.len();

    if n >= 3 {
        // Branchless version. Short circuiting doesn't help here.
        // Try to fully load all available ALUs by allowing as much ILP as possible.
        // The individual conditions change a lot, so reduce amount of jumping to
        // ease the pressure on the branch predictor.
        let run_n3 = unsafe { runs.get_unchecked(n.unchecked_sub(3)) };
        let run_n2 = unsafe { runs.get_unchecked(n.unchecked_sub(2)) };
        let run_n1 = unsafe { runs.get_unchecked(n.unchecked_sub(1)) };

        let top_run_zero = (run_n1.start == 0) as u8;
        let cond_a = (run_n2.len <= run_n1.len) as u8;
        let cond_b = (run_n3.len <= run_n2.len + run_n1.len) as u8;

        let cond_c = (n >= 4
            && unsafe { runs.get_unchecked(n.unchecked_sub(4)) }.len <= run_n3.len + run_n2.len)
            as u8;

        let cond: u8 = unsafe {
            top_run_zero
                .unchecked_add(cond_a)
                .unchecked_add(cond_b)
                .unchecked_add(cond_c)
        };

        if cond != 0 {
            let idx_add = (run_n3.len < run_n1.len) as u8;
            Some(unsafe { n.unchecked_sub(2 + idx_add as usize) })
        } else {
            None
        }
    } else if n == 2
        && (unsafe {
            runs.get_unchecked(1).start == 0
                || runs.get_unchecked(0).len <= runs.get_unchecked(1).len
        })
    {
        Some(n - 2)
    } else {
        None
    }
}

fn insertion_sort_remaining<T, F>(v: &mut [T], offset: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    // This is a logic but not a safety bug.
    debug_assert!(offset != 0 && offset <= len);

    if len < 2 || offset == 0 {
        return;
    }

    let arr_ptr = v.as_mut_ptr();

    // Shift each element of the unsorted region v[i..] as far left as is needed to make v sorted.
    for i in offset..len {
        unsafe {
            // SAFETY: we know i is at least 1 because offset is at least 1.
            // And we know len is at least 2.

            // There are three ways to implement insertion here:
            //
            // 1. Swap adjacent elements until the first one gets to its final destination.
            //    However, this way we copy data around more than is necessary. If elements are big
            //    structures (costly to copy), this method will be slow.
            //
            // 2. Iterate until the right place for the first element is found. Then shift the
            //    elements succeeding it to make room for it and finally place it into the
            //    remaining hole. This is a good method.
            //
            // 3. Copy the first element into a temporary variable. Iterate until the right place
            //    for it is found. As we go along, copy every traversed element into the slot
            //    preceding it. Finally, copy data from the temporary variable into the remaining
            //    hole. This method is very good. Benchmarks demonstrated slightly better
            //    performance than with the 2nd method.
            //
            // All methods were benchmarked, and the 3rd showed best results. So we chose that one.
            let i_ptr = arr_ptr.add(i);

            // It's important that we use i_ptr here. If this check is positive and we continue,
            // We want to make sure that no other copy of the value was seen by is_less.
            // Otherwise we would have to copy it back.
            if !is_less(&*i_ptr, &*i_ptr.sub(1)) {
                continue;
            }

            // It's important, that we use tmp for comparison from now on. As it is the value that
            // will be copied back. And notionally we could have created a divergence if we copy
            // back the wrong value.
            let tmp = mem::ManuallyDrop::new(ptr::read(i_ptr));
            // Intermediate state of the insertion process is always tracked by `hole`, which
            // serves two purposes:
            // 1. Protects integrity of `v` from panics in `is_less`.
            // 2. Fills the remaining hole in `v` in the end.
            //
            // Panic safety:
            //
            // If `is_less` panics at any point during the process, `hole` will get dropped and
            // fill the hole in `v` with `tmp`, thus ensuring that `v` still holds every object it
            // initially held exactly once.
            let mut hole = InsertionHole {
                src: &*tmp,
                dest: i_ptr.sub(1),
            };
            ptr::copy_nonoverlapping(hole.dest, i_ptr, 1);

            // SAFETY: We know i is at least 1.
            for j in (0..(i - 1)).rev() {
                let j_ptr = arr_ptr.add(j);
                if !is_less(&*tmp, &*j_ptr) {
                    break;
                }

                hole.dest = j_ptr;
                ptr::copy_nonoverlapping(hole.dest, j_ptr.add(1), 1);
            }
            // `hole` gets dropped and thus copies `tmp` into the remaining hole in `v`.
        }
    }
}

// Not actually any faster.
unsafe fn parity_merge_non_copy_safe<T: Debug, F, const LEN: usize>(
    src_ptr: *const T,
    dest_ptr: *mut T,
    is_less: &mut F,
) where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: the caller must guarantee that `src_ptr` and `dest_ptr` are valid for writes and
    // properly aligned. And they point to a contiguous owned region of memory each at least len
    // elements long. Also `src_ptr` and `dest_ptr` must not alias.

    // Setup call to parity_merge in a way that ensures two properties, that are not normally given
    // by parity_merge:
    // A) T is allowed to have non trivial destructors.
    // B) Uniqueness preservation for types with interior mutability.

    // Create another scratch area that is used to write into, and ignored when a panic occurs.
    let mut swap = mem::MaybeUninit::<[T; LEN]>::uninit();
    let swap_ptr = swap.as_mut_ptr() as *mut T;

    // Write sorted result into swap.
    // If is_less panics, dest_ptr which holds the initialized memory was not touched.
    //
    // is_less was called only using addresses from the original v slice to populate src_ptr.
    // src_ptr is now used as the sole source of addresses used for is_less.
    // Honestly something screwy is going on.
    parity_merge(src_ptr, swap_ptr, LEN, is_less);

    ptr::copy_nonoverlapping(swap_ptr, dest_ptr, LEN);
    mem::forget(swap);
}

#[test]
fn xx() {
    // let mut input = patterns::random_uniform(16, 10..100);
    // let mut input = (0..8).rev().collect::<Vec<i32>>();
    let mut input = vec![3, 7, 2, 4, 8, 0, 6, 1];
    sort_research_rs::new_stable_sort::sort(&mut input);

    // panic!();
}

// branchless main merge

fn merge() {
    let should_swap = is_less(&*right, &**left);

    let to_copy: *mut T = (((right as *mut T as usize) * should_swap as usize)
        + ((*left as *mut T as usize) * !should_swap as usize)) as *mut T;

    ptr::copy_nonoverlapping(to_copy, get_and_increment(out), 1);

    right = right.add(should_swap as usize);
    *left = left.add(!should_swap as usize);

    // ...

    let should_swap = is_less(&*right.offset(-1), &*left.offset(-1));

    *left = left.offset(-(should_swap as isize));
    *right = right.offset(-(!should_swap as isize));

    let to_copy: *mut T = (((*left as *mut T as usize) * should_swap as usize)
        + ((*right as *mut T as usize) * !should_swap as usize))
        as *mut T;

    ptr::copy_nonoverlapping(to_copy, decrement_and_get(&mut out), 1);
}

// What am I even doing
fn binary_search_sort<T, F>(v: &mut [T], offset: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // Find good spot to start insertion-sort via binary-search.
    // This minimizes the total amount of comparisons needed.
    //
    // We know that the first 16 elements (0..=15) are already sorted and Ord.

    let len = v.len();
    debug_assert!(offset <= len);

    let x = &v[offset];

    let mut search_for_pos = |low: usize, mid: usize, high: usize| {
        let is_less_pivot = is_less(&x, &v[mid - 1]);
        let pivot2 = if is_less_pivot { low } else { high };

        if is_less(&x, &v[pivot2 - 1]) {
            pivot2 - 1
        } else if is_less_pivot {
            mid - 1
        } else {
            offset
        }
    };

    // let start_pos = if offset >= 16 {
    //     search_for_pos(5, 9, 14)
    // } else if offset >= 8 {
    //     search_for_pos(3, 5, 7)
    // } else {
    //     offset
    // };
    let start_pos = offset;
    dbg!(offset);

    for i in (1..=start_pos).rev() {
        if !is_less(&x, &v[i - 1]) {
            // What would an algorithm be without rotate.
            dbg!(i);
            v[i..].rotate_right(1);
            break;
        }
    }
}

// Enable ILP while searching for the spots.
// Testing shows an average of 3 extra elements for a random input.
// match len {
//     17 => todo!(),
//     18 => todo!(),
//     19 => todo!(),
//     20 => todo!(),
//     _ => todo!(),
// }

// let mut search_for_pos = |offset: usize| {
//     let low = 5;
//     let mid = 9;
//     let high = 14;

//     let x = &v.get_unchecked(offset);

//     let is_less_pivot = is_less(&x, &v.get_unchecked(mid - 1));
//     let pivot2 = if is_less_pivot { low } else { high };

//     if is_less(&x, &v.get_unchecked(pivot2 - 1)) {
//         pivot2 - 1
//     } else if is_less_pivot {
//         mid - 1
//     } else {
//         offset
//     }
// };

// if len == 18 {
//     // let e1 = v.binary_search_by();
// } else {
//     insertion_sort_remaining(v, 16, is_less);
// }

// Fun search implementation with very few comparisons, but quite slow

fn binary_search_insert<T, F>(v: &mut [T], offset: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let x = &v[offset];

    let search_result = v[..offset].binary_search_by(|elem| {
        if is_less(elem, x) {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    let pos = match search_result {
        Ok(v) => v,
        Err(v) => v,
    };

    v[pos..=offset].rotate_right(1);
}

/// Merges already sorted v[offset..] into already sorted v[..offset].
fn merge_into_sorted<T, F>(v: &mut [T], offset: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: The caller has to ensure that offset <= len
    let len = v.len();

    // unsafe {
    //     let v_x = mem::transmute::<&mut [T], &mut [i32]>(v);
    //     dbg!(&v_x);
    // }

    for i in offset..len {
        binary_search_insert(v, i, is_less);
    }

    // unsafe {
    //     let v_x = mem::transmute::<&mut [T], &mut [i32]>(v);
    //     let mut v_copy = v_x.to_vec();
    //     v_copy.sort();
    //     assert_eq!(v_x, v_copy);
    //     dbg!(&v_x);
    // }
}

// Would have really thought this was faster.

/// Fast rotate_right(1) that works well for cheap to move types.
fn fast_rotate_right<T>(v: &mut [T]) {
    let len = v.len();

    if (len > 24) {
        debug_assert!(false); // Logic bug.
        return;
    }

    let mut tmp = mem::MaybeUninit::<[T; 24]>::uninit();

    // TODO ILP
    let arr_ptr = v.as_mut_ptr();
    let tmp_ptr = tmp.as_mut_ptr() as *mut T;

    // SAFETY: TODO
    unsafe {
        ptr::copy_nonoverlapping(arr_ptr, tmp_ptr, len);
        ptr::copy_nonoverlapping(tmp_ptr, arr_ptr.add(1), len - 1);
        ptr::copy_nonoverlapping(tmp_ptr.add(len - 1), arr_ptr, 1);
    }
}

// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
// performance impact.
#[inline(never)]
unsafe fn sort16_early_exit<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 16.
    debug_assert!(v.len() == 16);

    let arr_ptr = v.as_mut_ptr();

    // Custom sort network found with https://github.com/bertdobbelaere/SorterHunter
    // and with (0,1),(2,3),(4,5),(6,7),(8,9),(10,11),(12,13),(14,15),
    //          (1,2),(3,4),(5,6),(7,8),(9,10),(11,12),(13,14),
    // as FixedPrefix.
    // This allows efficient early exit if v is already or nearly sorted.

    // (0,1),(2,3),(4,5),(6,7),(8,9),(10,11),(12,13),(14,15),
    swap_if_less(arr_ptr, 0, 1, is_less);
    swap_if_less(arr_ptr, 2, 3, is_less);
    swap_if_less(arr_ptr, 4, 5, is_less);
    swap_if_less(arr_ptr, 6, 7, is_less);
    swap_if_less(arr_ptr, 8, 9, is_less);
    swap_if_less(arr_ptr, 10, 11, is_less);
    swap_if_less(arr_ptr, 12, 13, is_less);
    swap_if_less(arr_ptr, 14, 15, is_less);

    // (1,2),(3,4),(5,6),(7,8),(9,10),(11,12),(13,14),
    let should_swap_1_2 = is_less(&*arr_ptr.add(2), &*arr_ptr.add(1));
    let should_swap_3_4 = is_less(&*arr_ptr.add(4), &*arr_ptr.add(3));
    let should_swap_5_6 = is_less(&*arr_ptr.add(6), &*arr_ptr.add(5));
    let should_swap_7_8 = is_less(&*arr_ptr.add(8), &*arr_ptr.add(7));
    let should_swap_9_10 = is_less(&*arr_ptr.add(10), &*arr_ptr.add(9));
    let should_swap_11_12 = is_less(&*arr_ptr.add(12), &*arr_ptr.add(11));
    let should_swap_13_14 = is_less(&*arr_ptr.add(14), &*arr_ptr.add(13));

    // Do a single jump that is easy to predict.
    if (should_swap_1_2 as usize
        + should_swap_3_4 as usize
        + should_swap_5_6 as usize
        + should_swap_7_8 as usize
        + should_swap_9_10 as usize
        + should_swap_11_12 as usize
        + should_swap_13_14 as usize)
        == 0
    {
        // Do minimal comparisons if already sorted.
        return;
    }

    branchless_swap(arr_ptr.add(1), arr_ptr.add(2), should_swap_1_2);
    branchless_swap(arr_ptr.add(3), arr_ptr.add(4), should_swap_3_4);
    branchless_swap(arr_ptr.add(5), arr_ptr.add(6), should_swap_5_6);
    branchless_swap(arr_ptr.add(7), arr_ptr.add(8), should_swap_7_8);
    branchless_swap(arr_ptr.add(9), arr_ptr.add(10), should_swap_9_10);
    branchless_swap(arr_ptr.add(11), arr_ptr.add(12), should_swap_11_12);
    branchless_swap(arr_ptr.add(13), arr_ptr.add(14), should_swap_13_14);

    // (0,15),(3,11),(4,12),(0,7),(8,15),(4,8),(7,11),(1,5),(10,14),(0,3),(12,15),(2,6),(9,13),
    // (1,9),(6,14),(2,10),(5,13),(6,8),(7,9),(3,5),(10,12),(2,4),(11,13),(4,9),(6,11),(0,1),
    // (14,15),(5,9),(6,10),(3,7),(8,12),(2,7),(8,13),(4,7),(8,11),(1,3),(12,14),(5,6),(9,10),
    // (6,7),(8,9),(4,5),(10,11),(5,6),(9,10),(7,8),(2,3),(12,13),(3,4),(11,12)
    swap_if_less(arr_ptr, 0, 15, is_less);
    swap_if_less(arr_ptr, 3, 11, is_less);
    swap_if_less(arr_ptr, 4, 12, is_less);
    swap_if_less(arr_ptr, 0, 7, is_less);
    swap_if_less(arr_ptr, 8, 15, is_less);
    swap_if_less(arr_ptr, 4, 8, is_less);
    swap_if_less(arr_ptr, 7, 11, is_less);
    swap_if_less(arr_ptr, 1, 5, is_less);
    swap_if_less(arr_ptr, 10, 14, is_less);
    swap_if_less(arr_ptr, 0, 3, is_less);
    swap_if_less(arr_ptr, 12, 15, is_less);
    swap_if_less(arr_ptr, 2, 6, is_less);
    swap_if_less(arr_ptr, 9, 13, is_less);
    swap_if_less(arr_ptr, 1, 9, is_less);
    swap_if_less(arr_ptr, 6, 14, is_less);
    swap_if_less(arr_ptr, 2, 10, is_less);
    swap_if_less(arr_ptr, 5, 13, is_less);
    swap_if_less(arr_ptr, 6, 8, is_less);
    swap_if_less(arr_ptr, 7, 9, is_less);
    swap_if_less(arr_ptr, 3, 5, is_less);
    swap_if_less(arr_ptr, 10, 12, is_less);
    swap_if_less(arr_ptr, 2, 4, is_less);
    swap_if_less(arr_ptr, 11, 13, is_less);
    swap_if_less(arr_ptr, 4, 9, is_less);
    swap_if_less(arr_ptr, 6, 11, is_less);
    swap_if_less(arr_ptr, 0, 1, is_less);
    swap_if_less(arr_ptr, 14, 15, is_less);
    swap_if_less(arr_ptr, 5, 9, is_less);
    swap_if_less(arr_ptr, 6, 10, is_less);
    swap_if_less(arr_ptr, 3, 7, is_less);
    swap_if_less(arr_ptr, 8, 12, is_less);
    swap_if_less(arr_ptr, 2, 7, is_less);
    swap_if_less(arr_ptr, 8, 13, is_less);
    swap_if_less(arr_ptr, 4, 7, is_less);
    swap_if_less(arr_ptr, 8, 11, is_less);
    swap_if_less(arr_ptr, 1, 3, is_less);
    swap_if_less(arr_ptr, 12, 14, is_less);
    swap_if_less(arr_ptr, 5, 6, is_less);
    swap_if_less(arr_ptr, 9, 10, is_less);
    swap_if_less(arr_ptr, 6, 7, is_less);
    swap_if_less(arr_ptr, 8, 9, is_less);
    swap_if_less(arr_ptr, 4, 5, is_less);
    swap_if_less(arr_ptr, 10, 11, is_less);
    swap_if_less(arr_ptr, 5, 6, is_less);
    swap_if_less(arr_ptr, 9, 10, is_less);
    swap_if_less(arr_ptr, 7, 8, is_less);
    swap_if_less(arr_ptr, 2, 3, is_less);
    swap_if_less(arr_ptr, 12, 13, is_less);
    swap_if_less(arr_ptr, 3, 4, is_less);
    swap_if_less(arr_ptr, 11, 12, is_less);
}

// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
// performance impact.
#[inline(never)]
unsafe fn sort12_optimal<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 12.
    debug_assert!(v.len() == 12);

    let arr_ptr = v.as_mut_ptr();

    // Optimal sorting network see:
    // https://bertdobbelaere.github.io/sorting_networks_extended.html#N12L39D9

    swap_if_less(arr_ptr, 0, 8, is_less);
    swap_if_less(arr_ptr, 1, 7, is_less);
    swap_if_less(arr_ptr, 2, 6, is_less);
    swap_if_less(arr_ptr, 3, 11, is_less);
    swap_if_less(arr_ptr, 4, 10, is_less);
    swap_if_less(arr_ptr, 5, 9, is_less);
    swap_if_less(arr_ptr, 3, 5, is_less);
    swap_if_less(arr_ptr, 6, 8, is_less);
    swap_if_less(arr_ptr, 7, 10, is_less);
    swap_if_less(arr_ptr, 9, 11, is_less);
    swap_if_less(arr_ptr, 0, 1, is_less);
    swap_if_less(arr_ptr, 2, 9, is_less);
    swap_if_less(arr_ptr, 4, 7, is_less);
    swap_if_less(arr_ptr, 5, 6, is_less);
    swap_if_less(arr_ptr, 10, 11, is_less);
    swap_if_less(arr_ptr, 1, 3, is_less);
    swap_if_less(arr_ptr, 2, 7, is_less);
    swap_if_less(arr_ptr, 4, 9, is_less);
    swap_if_less(arr_ptr, 8, 10, is_less);
    swap_if_less(arr_ptr, 0, 1, is_less);
    swap_if_less(arr_ptr, 2, 3, is_less);
    swap_if_less(arr_ptr, 4, 5, is_less);
    swap_if_less(arr_ptr, 6, 7, is_less);
    swap_if_less(arr_ptr, 8, 9, is_less);
    swap_if_less(arr_ptr, 10, 11, is_less);
    swap_if_less(arr_ptr, 1, 2, is_less);
    swap_if_less(arr_ptr, 3, 5, is_less);
    swap_if_less(arr_ptr, 6, 8, is_less);
    swap_if_less(arr_ptr, 9, 10, is_less);
    swap_if_less(arr_ptr, 2, 4, is_less);
    swap_if_less(arr_ptr, 3, 6, is_less);
    swap_if_less(arr_ptr, 5, 8, is_less);
    swap_if_less(arr_ptr, 7, 9, is_less);
    swap_if_less(arr_ptr, 1, 2, is_less);
    swap_if_less(arr_ptr, 3, 4, is_less);
    swap_if_less(arr_ptr, 5, 6, is_less);
    swap_if_less(arr_ptr, 7, 8, is_less);
    swap_if_less(arr_ptr, 9, 10, is_less);
}

// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
// performance impact.
#[inline(never)]
unsafe fn sort16_optimal<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 16.
    debug_assert!(v.len() == 16);

    let arr_ptr = v.as_mut_ptr();

    // Optimal sorting network see:
    // https://bertdobbelaere.github.io/sorting_networks_extended.html#N16L60D10

    swap_if_less(arr_ptr, 0, 13, is_less);
    swap_if_less(arr_ptr, 1, 12, is_less);
    swap_if_less(arr_ptr, 2, 15, is_less);
    swap_if_less(arr_ptr, 3, 14, is_less);
    swap_if_less(arr_ptr, 4, 8, is_less);
    swap_if_less(arr_ptr, 5, 6, is_less);
    swap_if_less(arr_ptr, 7, 11, is_less);
    swap_if_less(arr_ptr, 9, 10, is_less);
    swap_if_less(arr_ptr, 0, 5, is_less);
    swap_if_less(arr_ptr, 1, 7, is_less);
    swap_if_less(arr_ptr, 2, 9, is_less);
    swap_if_less(arr_ptr, 3, 4, is_less);
    swap_if_less(arr_ptr, 6, 13, is_less);
    swap_if_less(arr_ptr, 8, 14, is_less);
    swap_if_less(arr_ptr, 10, 15, is_less);
    swap_if_less(arr_ptr, 11, 12, is_less);
    swap_if_less(arr_ptr, 0, 1, is_less);
    swap_if_less(arr_ptr, 2, 3, is_less);
    swap_if_less(arr_ptr, 4, 5, is_less);
    swap_if_less(arr_ptr, 6, 8, is_less);
    swap_if_less(arr_ptr, 7, 9, is_less);
    swap_if_less(arr_ptr, 10, 11, is_less);
    swap_if_less(arr_ptr, 12, 13, is_less);
    swap_if_less(arr_ptr, 14, 15, is_less);
    swap_if_less(arr_ptr, 0, 2, is_less);
    swap_if_less(arr_ptr, 1, 3, is_less);
    swap_if_less(arr_ptr, 4, 10, is_less);
    swap_if_less(arr_ptr, 5, 11, is_less);
    swap_if_less(arr_ptr, 6, 7, is_less);
    swap_if_less(arr_ptr, 8, 9, is_less);
    swap_if_less(arr_ptr, 12, 14, is_less);
    swap_if_less(arr_ptr, 13, 15, is_less);
    swap_if_less(arr_ptr, 1, 2, is_less);
    swap_if_less(arr_ptr, 3, 12, is_less);
    swap_if_less(arr_ptr, 4, 6, is_less);
    swap_if_less(arr_ptr, 5, 7, is_less);
    swap_if_less(arr_ptr, 8, 10, is_less);
    swap_if_less(arr_ptr, 9, 11, is_less);
    swap_if_less(arr_ptr, 13, 14, is_less);
    swap_if_less(arr_ptr, 1, 4, is_less);
    swap_if_less(arr_ptr, 2, 6, is_less);
    swap_if_less(arr_ptr, 5, 8, is_less);
    swap_if_less(arr_ptr, 7, 10, is_less);
    swap_if_less(arr_ptr, 9, 13, is_less);
    swap_if_less(arr_ptr, 11, 14, is_less);
    swap_if_less(arr_ptr, 2, 4, is_less);
    swap_if_less(arr_ptr, 3, 6, is_less);
    swap_if_less(arr_ptr, 9, 12, is_less);
    swap_if_less(arr_ptr, 11, 13, is_less);
    swap_if_less(arr_ptr, 3, 5, is_less);
    swap_if_less(arr_ptr, 6, 8, is_less);
    swap_if_less(arr_ptr, 7, 9, is_less);
    swap_if_less(arr_ptr, 10, 12, is_less);
    swap_if_less(arr_ptr, 3, 4, is_less);
    swap_if_less(arr_ptr, 5, 6, is_less);
    swap_if_less(arr_ptr, 7, 8, is_less);
    swap_if_less(arr_ptr, 9, 10, is_less);
    swap_if_less(arr_ptr, 11, 12, is_less);
    swap_if_less(arr_ptr, 6, 7, is_less);
    swap_if_less(arr_ptr, 8, 9, is_less);
}

// Not actually stable

/// Sort the first 2 elements of v.
unsafe fn sort2<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v is at least len 2.
    debug_assert!(v.len() == 2);

    let arr_ptr = v.as_mut_ptr();

    swap_if_less(arr_ptr, 0, 1, is_less);
}

/// Sort the first 3 elements of v.
unsafe fn sort3<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v is at least len 3.
    debug_assert!(v.len() == 3);

    let arr_ptr = v.as_mut_ptr();

    swap_if_less(arr_ptr, 0, 1, is_less);
    swap_if_less(arr_ptr, 1, 2, is_less);
    swap_if_less(arr_ptr, 0, 1, is_less);
}

/// Sort the first 4 elements of v without any jump instruction.
unsafe fn sort4<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v is at least len 4.
    debug_assert!(v.len() == 4);

    let arr_ptr = v.as_mut_ptr();

    swap_if_less(arr_ptr, 0, 1, is_less);
    swap_if_less(arr_ptr, 2, 3, is_less);

    // PANIC SAFETY: if is_less panics, no scratch memory was created and the slice should still be
    // in a well defined state, without duplicates.
    if is_less(&*arr_ptr.add(2), &*arr_ptr.add(1)) {
        ptr::swap_nonoverlapping(arr_ptr.add(1), arr_ptr.add(2), 1);

        swap_if_less(arr_ptr, 0, 1, is_less);
        swap_if_less(arr_ptr, 2, 3, is_less);
        swap_if_less(arr_ptr, 1, 2, is_less);
    }
}

// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
// performance impact.
#[inline(never)]
unsafe fn sort8<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 8.
    debug_assert!(v.len() == 8);

    let arr_ptr = v.as_mut_ptr();

    // Custom sort network found with https://github.com/bertdobbelaere/SorterHunter
    // With custom prefix to enable early exit.

    // (0,1),(2,3),(4,5),(6,7)
    swap_if_less(arr_ptr, 0, 1, is_less);
    swap_if_less(arr_ptr, 2, 3, is_less);
    swap_if_less(arr_ptr, 4, 5, is_less);
    swap_if_less(arr_ptr, 6, 7, is_less);

    // (1,2),(3,4),(5,6)
    swap_if_less(arr_ptr, 1, 2, is_less);
    swap_if_less(arr_ptr, 3, 4, is_less);
    swap_if_less(arr_ptr, 5, 6, is_less);

    // let should_swap_1_2 = is_less(&*arr_ptr.add(2), &*arr_ptr.add(1));
    // let should_swap_3_4 = is_less(&*arr_ptr.add(4), &*arr_ptr.add(3));
    // let should_swap_5_6 = is_less(&*arr_ptr.add(6), &*arr_ptr.add(5));

    // // Do a single jump that is easy to predict.
    // if (should_swap_1_2 as usize + should_swap_3_4 as usize + should_swap_5_6 as usize) == 0 {
    //     // Do minimal comparisons if already sorted.
    //     return;
    // }

    // branchless_swap(arr_ptr.add(1), arr_ptr.add(2), should_swap_1_2);
    // branchless_swap(arr_ptr.add(3), arr_ptr.add(4), should_swap_3_4);
    // branchless_swap(arr_ptr.add(5), arr_ptr.add(6), should_swap_5_6);

    // (0,7),(1,5),(2,6),(0,3),(4,7),(0,1),(6,7),(2,4),(3,5),(2,3),(4,5),(1,2),(5,6),(3,4)
    swap_if_less(arr_ptr, 0, 7, is_less);
    swap_if_less(arr_ptr, 1, 5, is_less);
    swap_if_less(arr_ptr, 2, 6, is_less);
    swap_if_less(arr_ptr, 0, 3, is_less);
    swap_if_less(arr_ptr, 4, 7, is_less);
    swap_if_less(arr_ptr, 0, 1, is_less);
    swap_if_less(arr_ptr, 6, 7, is_less);
    swap_if_less(arr_ptr, 2, 4, is_less);
    swap_if_less(arr_ptr, 3, 5, is_less);
    swap_if_less(arr_ptr, 2, 3, is_less);
    swap_if_less(arr_ptr, 4, 5, is_less);
    swap_if_less(arr_ptr, 1, 2, is_less);
    swap_if_less(arr_ptr, 5, 6, is_less);
    swap_if_less(arr_ptr, 3, 4, is_less);
}

// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
// performance impact.
#[inline(never)]
unsafe fn sort12_plus<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 12.
    let len = v.len();
    debug_assert!(len >= 12);

    // Do some checks to enable minimal comparisons for already sorted inputs.
    let arr_ptr = v.as_mut_ptr();

    let should_swap_0_1 = is_less(&*arr_ptr.add(1), &*arr_ptr.add(0));
    let should_swap_2_1 = is_less(&*arr_ptr.add(2), &*arr_ptr.add(1));
    let should_swap_3_2 = is_less(&*arr_ptr.add(3), &*arr_ptr.add(2));
    let should_swap_4_3 = is_less(&*arr_ptr.add(4), &*arr_ptr.add(3));

    let swap_count = should_swap_0_1 as usize
        + should_swap_2_1 as usize
        + should_swap_3_2 as usize
        + should_swap_4_3 as usize;

    // The heuristic here is that if the first 5 elements are already sorted, chances are it is
    // already sorted, and we dispatch into the potentially slower version that checks that.

    if swap_count == 0 {
        // Potentially already sorted.
        insertion_sort_remaining(v, 4, is_less);
    } else if swap_count == 4 {
        // Potentially reversed.
        let mut rev_i = 4;
        let end = len - 1;
        while rev_i < end {
            if !is_less(&*arr_ptr.add(rev_i + 1), &*arr_ptr.add(rev_i)) {
                break;
            }
            rev_i += 1;
        }
        v[..rev_i].reverse();
        insertion_sort_remaining(v, rev_i, is_less);
    } else {
        if len < 20 {
            // Optimal sorting networks like sort12_optimal and sort16_optimal would save up 2x
            // runtime here, but they would only be applicable to sizes 12..=19. But would incur a
            // sizable binary overhead that doesn't seem worth it.

            sort8(&mut v[..8], is_less);

            if len < 16 {
                sort4(&mut v[8..12], is_less);
                insertion_sort_remaining(&mut v[8..], 4, is_less);
            } else {
                sort8(&mut v[8..16], is_less);
                insertion_sort_remaining(&mut v[8..], 8, is_less);
            }

            // SAFETY: The shorter side will always be at most 8 long. Because 0..8.len() == 8
            let mut swap = mem::MaybeUninit::<[T; 8]>::uninit();
            let swap_ptr = swap.as_mut_ptr() as *mut T;
            merge(v, 8, swap_ptr, is_less);
        } else {
            sort20_optimal(&mut v[..20], is_less);
            insertion_sort_remaining(v, 20, is_less);
        }
    }
}

// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
// performance impact.
#[inline(never)]
unsafe fn sort20_optimal<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 20.
    debug_assert!(v.len() == 20);

    let arr_ptr = v.as_mut_ptr();

    // Optimal sorting network see:
    // https://bertdobbelaere.github.io/sorting_networks_extended.html#N20L91D12

    swap_if_less(arr_ptr, 0, 3, is_less);
    swap_if_less(arr_ptr, 1, 7, is_less);
    swap_if_less(arr_ptr, 2, 5, is_less);
    swap_if_less(arr_ptr, 4, 8, is_less);
    swap_if_less(arr_ptr, 6, 9, is_less);
    swap_if_less(arr_ptr, 10, 13, is_less);
    swap_if_less(arr_ptr, 11, 15, is_less);
    swap_if_less(arr_ptr, 12, 18, is_less);
    swap_if_less(arr_ptr, 14, 17, is_less);
    swap_if_less(arr_ptr, 16, 19, is_less);
    swap_if_less(arr_ptr, 0, 14, is_less);
    swap_if_less(arr_ptr, 1, 11, is_less);
    swap_if_less(arr_ptr, 2, 16, is_less);
    swap_if_less(arr_ptr, 3, 17, is_less);
    swap_if_less(arr_ptr, 4, 12, is_less);
    swap_if_less(arr_ptr, 5, 19, is_less);
    swap_if_less(arr_ptr, 6, 10, is_less);
    swap_if_less(arr_ptr, 7, 15, is_less);
    swap_if_less(arr_ptr, 8, 18, is_less);
    swap_if_less(arr_ptr, 9, 13, is_less);
    swap_if_less(arr_ptr, 0, 4, is_less);
    swap_if_less(arr_ptr, 1, 2, is_less);
    swap_if_less(arr_ptr, 3, 8, is_less);
    swap_if_less(arr_ptr, 5, 7, is_less);
    swap_if_less(arr_ptr, 11, 16, is_less);
    swap_if_less(arr_ptr, 12, 14, is_less);
    swap_if_less(arr_ptr, 15, 19, is_less);
    swap_if_less(arr_ptr, 17, 18, is_less);
    swap_if_less(arr_ptr, 1, 6, is_less);
    swap_if_less(arr_ptr, 2, 12, is_less);
    swap_if_less(arr_ptr, 3, 5, is_less);
    swap_if_less(arr_ptr, 4, 11, is_less);
    swap_if_less(arr_ptr, 7, 17, is_less);
    swap_if_less(arr_ptr, 8, 15, is_less);
    swap_if_less(arr_ptr, 13, 18, is_less);
    swap_if_less(arr_ptr, 14, 16, is_less);
    swap_if_less(arr_ptr, 0, 1, is_less);
    swap_if_less(arr_ptr, 2, 6, is_less);
    swap_if_less(arr_ptr, 7, 10, is_less);
    swap_if_less(arr_ptr, 9, 12, is_less);
    swap_if_less(arr_ptr, 13, 17, is_less);
    swap_if_less(arr_ptr, 18, 19, is_less);
    swap_if_less(arr_ptr, 1, 6, is_less);
    swap_if_less(arr_ptr, 5, 9, is_less);
    swap_if_less(arr_ptr, 7, 11, is_less);
    swap_if_less(arr_ptr, 8, 12, is_less);
    swap_if_less(arr_ptr, 10, 14, is_less);
    swap_if_less(arr_ptr, 13, 18, is_less);
    swap_if_less(arr_ptr, 3, 5, is_less);
    swap_if_less(arr_ptr, 4, 7, is_less);
    swap_if_less(arr_ptr, 8, 10, is_less);
    swap_if_less(arr_ptr, 9, 11, is_less);
    swap_if_less(arr_ptr, 12, 15, is_less);
    swap_if_less(arr_ptr, 14, 16, is_less);
    swap_if_less(arr_ptr, 1, 3, is_less);
    swap_if_less(arr_ptr, 2, 4, is_less);
    swap_if_less(arr_ptr, 5, 7, is_less);
    swap_if_less(arr_ptr, 6, 10, is_less);
    swap_if_less(arr_ptr, 9, 13, is_less);
    swap_if_less(arr_ptr, 12, 14, is_less);
    swap_if_less(arr_ptr, 15, 17, is_less);
    swap_if_less(arr_ptr, 16, 18, is_less);
    swap_if_less(arr_ptr, 1, 2, is_less);
    swap_if_less(arr_ptr, 3, 4, is_less);
    swap_if_less(arr_ptr, 6, 7, is_less);
    swap_if_less(arr_ptr, 8, 9, is_less);
    swap_if_less(arr_ptr, 10, 11, is_less);
    swap_if_less(arr_ptr, 12, 13, is_less);
    swap_if_less(arr_ptr, 15, 16, is_less);
    swap_if_less(arr_ptr, 17, 18, is_less);
    swap_if_less(arr_ptr, 2, 3, is_less);
    swap_if_less(arr_ptr, 4, 6, is_less);
    swap_if_less(arr_ptr, 5, 8, is_less);
    swap_if_less(arr_ptr, 7, 9, is_less);
    swap_if_less(arr_ptr, 10, 12, is_less);
    swap_if_less(arr_ptr, 11, 14, is_less);
    swap_if_less(arr_ptr, 13, 15, is_less);
    swap_if_less(arr_ptr, 16, 17, is_less);
    swap_if_less(arr_ptr, 4, 5, is_less);
    swap_if_less(arr_ptr, 6, 8, is_less);
    swap_if_less(arr_ptr, 7, 10, is_less);
    swap_if_less(arr_ptr, 9, 12, is_less);
    swap_if_less(arr_ptr, 11, 13, is_less);
    swap_if_less(arr_ptr, 14, 15, is_less);
    swap_if_less(arr_ptr, 3, 4, is_less);
    swap_if_less(arr_ptr, 5, 6, is_less);
    swap_if_less(arr_ptr, 7, 8, is_less);
    swap_if_less(arr_ptr, 9, 10, is_less);
    swap_if_less(arr_ptr, 11, 12, is_less);
    swap_if_less(arr_ptr, 13, 14, is_less);
    swap_if_less(arr_ptr, 15, 16, is_less);
}

// Only worth for 20, not for 16.
fn x() {
    if qualifies_for_branchless_sort::<T>() && end >= 23 && start_end_diff <= 6 {
        // For random inputs on average how many elements are naturally already sorted
        // (start_end_diff) will be relatively small. And it's faster to avoid a merge operation
        // between the newly sorted elements on the left by the sort network and the already sorted
        // elements. Instead if there are 3 or fewer already sorted elements they get merged by
        // participating in the sort network. This wastes the information that they are already
        // sorted, but extra branching is not worth it.
        let is_small_pre_sorted = start_end_diff <= 3;

        start = if is_small_pre_sorted {
            end - 20
        } else {
            start_found - 17
        };

        // SAFETY: start >= 0 && start + 20 <= end
        unsafe {
            // Use an optimal sorting network here instead of some hybrid network with early exit.
            // If the input is already sorted the previous adaptive analysis path of Timsort ought
            // to have found it. So we prefer minimizing the total amount of comparisons, which are
            // user provided and may be of arbitrary cost.
            sort20_optimal(&mut v[start..(start + 20)], is_less);
        }

        // For most patterns this branch should have good prediction accuracy.
        if !is_small_pre_sorted {
            insertion_sort_remaining(&mut v[start..end], 20, is_less);
        }
    }
}

// Easier done in place
#[inline]
unsafe fn sort2_idx<T, F>(v: &mut [T], a: usize, b: usize, is_less: &mut F) -> bool
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 2. TODO idx.
    debug_assert!(v.len() == 2 && a != b);

    let arr_ptr = v.as_mut_ptr();

    if qualifies_for_branchless_sort::<T>() {
        swap_if_less(arr_ptr, a, b, is_less)
    } else {
        let should_swap = is_less(&*arr_ptr.add(b), &*arr_ptr.add(a));
        if should_swap {
            ptr::swap_nonoverlapping(arr_ptr.add(a), arr_ptr.add(b), 1);
        }
        should_swap
    }
}

#[inline]
unsafe fn sort3_idx<T, F>(
    v: &mut [T],
    a: &mut usize,
    b: &mut usize,
    c: &mut usize,
    is_less: &mut F,
) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() >= 3. TODO idx.
    debug_assert!(v.len() == 3 && a != b && a != c);

    let swaps = 0;

    swaps += sort2_idx(v, a, b, is_less) as usize;
    swaps += sort2_idx(v, b, c, is_less) as usize;
    swaps += sort2_idx(v, a, b, is_less) as usize;

    swaps
}

// Debug func
// if len <= 34 {
//     unsafe {
//         // FIXME
//         let xx = mem::transmute::<&mut [T], &mut [i32]>(v);
//         println!("{xx:?}");

//         let xx = mem::transmute::<&T, &i32>(&v[pivot]);
//         println!("v[pivot]: {xx}");
//     }
// }

// A beautiful popsicle, sniff :( but slow
/// Partitions `v` into elements smaller than `pivot`, followed by elements greater than or equal
/// to `pivot`.
///
/// Returns the number of elements smaller than `pivot`.
///
/// Novel partitioning algorithm designed to enable as much ILP as possible.
/// TODO investigate variant that maintains relative order.
fn partition_in_blocks<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    #[inline]
    pub unsafe fn swap_if_less_proxy<T>(arr_ptr: *mut T, comp_result_ptr: *mut u8, offset: usize) {
        // SAFETY: The caller must ensure that TODO
        unsafe {
            // Note, it's crucial that this check be performed without a branch.
            let should_swap = *comp_result_ptr == 0;

            let a_ptr = arr_ptr;
            let b_ptr = arr_ptr.add(offset);

            let comp_a_ptr = comp_result_ptr;
            let comp_b_ptr = comp_result_ptr.add(offset);

            branchless_swap(a_ptr, b_ptr, should_swap);
            branchless_swap(comp_a_ptr, comp_b_ptr, should_swap);
        }
    }

    /// Using the comparison results of comp_results, where 1 -> is less and 0 -> equal or more.
    /// Partition arr_ptr[0..4] so that all values that are less are on the left side.
    /// Eg. comp_results [0, 1, 0, 1] -> [1, 1, 0, 0]
    /// Note, it does not preserve input order, in the above example 0 and 0 could have been swapped.
    #[inline]
    pub unsafe fn partition4_proxy<T>(arr_ptr: *mut T, comp_results: &mut [u8; 4], i: usize) {
        // SAFETY: The caller must ensure that 4 elements exist in arr_ptr.add(i).
        unsafe {
            let i_ptr = arr_ptr.add(i);
            let c_ptr = comp_results.as_mut_ptr() as *mut u8;

            // Optimal 4 element sorting network.
            swap_if_less_proxy(i_ptr.add(0), c_ptr.add(0), 2);
            swap_if_less_proxy(i_ptr.add(1), c_ptr.add(1), 2);
            swap_if_less_proxy(i_ptr.add(0), c_ptr.add(0), 1);
            swap_if_less_proxy(i_ptr.add(2), c_ptr.add(2), 1);
            swap_if_less_proxy(i_ptr.add(1), c_ptr.add(1), 1);
        }
    }

    // Scan with two windows, from left and right.
    // Rough steps:
    // 1. Store result of is_less for each window element.
    // 2. Determine if more values need to move left or right of the pivot.
    // 3. Use temporary memory to efficiently swap elements.
    // 4. Adjust windows as required
    // 5. Repeat.

    let len = v.len();

    // SAFETY: Don't change this value without adjusting the relevant parts of the code below that
    // are not automatically adjusted to another WINDOW_SIZE, such as write_comp_result calls.
    const WINDOW_SIZE: usize = 4;
    // const BLOCK_SIZE: usize = WINDOW_SIZE * 2;

    let write_comp_result =
        |arr_ptr: *mut T, comp_ptr: *mut u8, i: usize, offset: usize, is_less: &mut F| unsafe {
            // SAFETY: The caller must ensure that i + offset is inbounds for v and offset is <
            // WINDOW_SIZE.
            unsafe {
                comp_ptr
                    .add(offset)
                    .write(is_less(&*arr_ptr.add(i + offset), pivot) as u8);
            }
        };

    let arr_ptr = v.as_mut_ptr();

    let mut l_comp_results = [0u8; WINDOW_SIZE];
    let mut r_comp_results = [0u8; WINDOW_SIZE];

    // The number of elements smaller than pivot.
    let mut smaller_total = 0;

    // // Worst case WINDOW_SIZE elements have to move from the right window to the left side.
    // let mut swap = mem::MaybeUninit::<[T; WINDOW_SIZE]>::uninit();
    // let swap_ptr = swap.as_mut_ptr() as *mut T;

    // let mut index_stack = [0usize; BLOCK_SIZE];
    // let mut index_stack_ptr = index_stack.as_mut_ptr() as *mut usize;

    let mut l_window_i = 0;
    let mut r_window_i = len.saturating_sub(WINDOW_SIZE);

    // SAFETY: Ensure there can always be two full windows between the two windows.
    while ((r_window_i + WINDOW_SIZE) - l_window_i) >= WINDOW_SIZE * 4 {
        // First perform the is_less calls which can panic. So that later swapping with temporary
        // elements can be done without special drop guards. Additionally this allows further
        // comparisons with a fixed cheap cost regardless of the cost of is_less.

        // Loop unrolled to allow ILP.
        // SAFETY: we checked that i is in bounds and comp_results is large enough.
        unsafe {
            let l_comp_ptr = l_comp_results.as_mut_ptr() as *mut u8;
            write_comp_result(arr_ptr, l_comp_ptr, l_window_i, 0, is_less);
            write_comp_result(arr_ptr, l_comp_ptr, l_window_i, 1, is_less);
            write_comp_result(arr_ptr, l_comp_ptr, l_window_i, 2, is_less);
            write_comp_result(arr_ptr, l_comp_ptr, l_window_i, 3, is_less);

            let r_comp_ptr = r_comp_results.as_mut_ptr() as *mut u8;
            write_comp_result(arr_ptr, r_comp_ptr, r_window_i, 0, is_less);
            write_comp_result(arr_ptr, r_comp_ptr, r_window_i, 1, is_less);
            write_comp_result(arr_ptr, r_comp_ptr, r_window_i, 2, is_less);
            write_comp_result(arr_ptr, r_comp_ptr, r_window_i, 3, is_less);
        }

        // Swap elements around in each window so that the values that are less than pivot are on
        // the left side.
        //
        // SAFETY: TODO
        unsafe {
            partition4_proxy(arr_ptr, &mut l_comp_results, l_window_i);
            partition4_proxy(arr_ptr, &mut r_comp_results, r_window_i);
        }

        // TODO check perf impact of doing this before partition4_proxy.
        let sum_left = l_comp_results.iter().sum::<u8>() as usize;
        let sum_right = r_comp_results.iter().sum::<u8>() as usize;

        let in_order_left = sum_left;
        let in_order_right = WINDOW_SIZE - sum_right;

        let out_of_order_left = WINDOW_SIZE - sum_left;
        let out_of_order_right = sum_right;

        // Don't increase smaller_total as part of write_comp_result to avoid a memory dependency.
        smaller_total += in_order_left + out_of_order_right;

        // let window_range = unsafe {
        //     // FIXME
        //     let v_i32 = mem::transmute::<&[T], &[i32]>(v);

        //     let left_window = &v_i32[l_window_i..(l_window_i + WINDOW_SIZE)];
        //     let right_window = &v_i32[r_window_i..(r_window_i + WINDOW_SIZE)];

        //     let save_window_left = &v_i32
        //         [(l_window_i + in_order_left)..(l_window_i + in_order_left + out_of_order_left)];
        //     let move_left_from_right = &v_i32[r_window_i..(r_window_i + out_of_order_right)];
        //     let overwrite_right_from_save =
        //         &v_i32[(r_window_i - (in_order_right + out_of_order_left) + WINDOW_SIZE)
        //             ..(r_window_i - (in_order_right + out_of_order_left)
        //                 + WINDOW_SIZE
        //                 + out_of_order_left)];

        //     let window_range = l_window_i..(r_window_i + WINDOW_SIZE);
        //     println!("{:?}", &v_i32[window_range.clone()]);
        //     println!(
        //         "{:?} {:?} | {:?} {:?} {:?}",
        //         left_window,
        //         right_window,
        //         save_window_left,
        //         move_left_from_right,
        //         overwrite_right_from_save
        //     );

        //     window_range
        // };

        // Now that both sides look like this, eg:
        // left [1, 1, 1, 0] ... right [1, 1, 0, 0]
        // TODO explain.
        //
        // SAFETY: TODO
        unsafe {
            let l_swap_ptr = arr_ptr.add(l_window_i + in_order_left);
            let r_swap_ptr = arr_ptr.add(r_window_i - in_order_right);

            // Combined swap and rotate operation.
            // Always swap fixed size for good code gen.
            ptr::swap_nonoverlapping(l_swap_ptr.add(0), r_swap_ptr.add(3), 1);
            ptr::swap_nonoverlapping(l_swap_ptr.add(1), r_swap_ptr.add(2), 1);
            ptr::swap_nonoverlapping(l_swap_ptr.add(2), r_swap_ptr.add(1), 1);
            ptr::swap_nonoverlapping(l_swap_ptr.add(3), r_swap_ptr.add(0), 1);

            // Now it looks like this, where _ is unknown if less than pivot.
            // left [1, 1, 1, 1, 1, _, _] right [_, _, _, 0, 0, 0]
        }

        l_window_i += in_order_left + out_of_order_right;
        r_window_i -= in_order_right + out_of_order_left;

        // unsafe {
        //     // FIXME
        //     let v_i32 = mem::transmute::<&[T], &[i32]>(v);
        //     println!("{:?}", &v_i32[window_range.clone()]);
        //     println!();
        // }
    }

    // smaller_total

    // Partition the remaining elements between the last windows with a simple algorithm.
    let mut l = l_window_i;
    let mut r = cmp::min(r_window_i + WINDOW_SIZE, len);
    loop {
        // SAFETY: The unsafety below involves indexing an array.
        // For the first one: We already do the bounds checking here with `l < r`.
        // For the second one: We initially have `l == 0` and `r == v.len()` and we checked that `l < r` at every indexing operation.
        //                     From here we know that `r` must be at least `r == l` which was shown to be valid from the first one.
        unsafe {
            // Find the first element less than the pivot.
            while l < r && is_less(v.get_unchecked(l), pivot) {
                l += 1;
            }

            // Find the last element equal or more to the pivot.
            while l < r && !is_less(v.get_unchecked(r - 1), pivot) {
                r -= 1;
            }

            // Are we done?
            if l >= r {
                break;
            }

            // Swap the found pair of out-of-order elements.
            r -= 1;
            let ptr = v.as_mut_ptr();
            ptr::swap(ptr.add(l), ptr.add(r));
            l += 1;
        }
    }

    (smaller_total + l) - l_window_i
    // let r = cmp::min(r_window_i + WINDOW_SIZE, len);
    // let rest = partition_in_blocks_old(&mut v[l..r], pivot, is_less);

    // let l = l_window_i;
    // smaller_total + rest
}

// Cute and slow sigh
fn partition_in_blocks<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    if len < 2 {
        if len == 0 {
            return 0;
        }
        // SAFETY: We know v has len 1.
        unsafe {
            return is_less(v.get_unchecked(0), pivot) as usize;
        }
    }

    // let mut smaller_total = 0;

    let arr_ptr = v.as_mut_ptr();

    // SAFETY: TODO
    unsafe {
        let mut l_ptr = arr_ptr;
        let mut r_ptr = arr_ptr.add(len - 1);

        while r_ptr > l_ptr {
            let l_less = is_less(&*l_ptr, pivot);

            let should_swap = !l_less;
            branchless_swap(l_ptr, r_ptr, should_swap);

            // Only increase l_ptr if it was not swapped.
            l_ptr = l_ptr.add(l_less as usize);

            // Only decrease r_ptr if it was swapped.
            r_ptr = r_ptr.sub(should_swap as usize);
        }

        // Do final fixup.
        l_ptr = l_ptr.add(is_less(&*l_ptr, pivot) as usize);

        intrinsics::ptr_offset_from_unsigned(l_ptr, arr_ptr)
    }
}

// Still slower than BlockQuicksort partition !!
fn partition_in_blocks<T, F>(
    v: &mut [T],
    pivot: &T,
    buf_left: *mut T,
    buf_right: *mut T,
    is_less: &mut F,
) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // let's just use memory hihi :)
    let len = v.len();

    let mut left_ptr = buf_left;
    let mut right_ptr = buf_right;

    for elem in v.iter() {
        let belong_on_left = is_less(elem, pivot);
        let target_ptr = if belong_on_left { left_ptr } else { right_ptr };

        // SAFETY: TODO
        unsafe {
            ptr::copy_nonoverlapping(elem, target_ptr, 1);

            left_ptr = left_ptr.add(belong_on_left as usize);
            right_ptr = right_ptr.add(!belong_on_left as usize);
        }
    }

    // SAFETY: TODO
    let smaller_count = unsafe { intrinsics::ptr_offset_from_unsigned(left_ptr, buf_left) };

    // SAFETY: TODO
    unsafe {
        let arr_ptr = v.as_mut_ptr();
        ptr::copy_nonoverlapping(buf_left, arr_ptr, smaller_count);
        ptr::copy_nonoverlapping(buf_right, arr_ptr.add(smaller_count), len - smaller_count);
    }

    smaller_count
}

// partition attempts

#![allow(unused)]

use core::cmp;
use core::intrinsics;
use core::mem;
use core::ptr;
use core::simd;

use crate::unstable::rust_ipn::branchless_swap;

partition_impl!("ilp_partition");

const OFFSET_SENTINEL: u8 = u8::MAX;

unsafe fn collect_offsets_16<T, F>(v: &[T], pivot: &T, offset_lane_ptr: *mut u8, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    debug_assert!(v.len() == BLOCK_SIZE);

    // SAFETY: offset_lane_ptr must be able to hold 16 elements.

    const LANES: usize = 4;
    const BLOCK_SIZE: usize = LANES * LANES;

    let mut offset_lane_ptrs = [ptr::null_mut::<u8>(); LANES];

    let check_offset = |l_ptr: *const T, i: usize, offsets_ptr: *mut u8, is_less: &mut F| unsafe {};

    let arr_ptr = v.as_ptr();

    for lane in 0..LANES {
        let offsets_base_ptr = offset_lane_ptr.add(lane * LANES);
        offset_lane_ptrs[lane] = offsets_base_ptr;

        // This inner loop should be unfolded by the optimizer.
        for lane_offset in 0..LANES {
            let offset = (LANES * lane) + lane_offset;
            let is_r_elem = !is_less(&*arr_ptr.add(offset), pivot);
            offset_lane_ptrs[lane].write(offset as u8);
            offset_lane_ptrs[lane] = offset_lane_ptrs[lane].add(is_r_elem as usize);
        }
    }
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
unsafe fn collect_offsets_n<const N: usize, T, F>(v: &[T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    debug_assert!(v.len() == N && N >= 16 && N % 16 == 0 && N < u8::MAX as usize);

    use core::arch::x86_64;

    // let mut offsets = [OFFSET_SENTINEL; N];
    // let offsets_ptr = offsets.as_mut_ptr();

    let arr_ptr = v.as_ptr();

    // for offset in 0..(N as u8) {
    //     let is_r_elem = !is_less(&*arr_ptr.add(offset as usize), pivot);
    //     offsets_ptr.write(offset);
    //     offsets_ptr = offsets_ptr.add(is_r_elem as usize);
    // }
    // let sum = intrinsics::ptr_offset_from_unsigned(offsets_ptr, offsets.as_mut_ptr());

    let mask = x86_64::__m128i::from(simd::u8x16::splat(u8::MAX));

    let mut sum = 0;
    let mut i = 0;
    while i < N {
        let mut offsets = simd::u8x16::splat(u8::MAX);
        collect_offsets_16(&v[i..], pivot, offsets.as_mut_array().as_mut_ptr(), is_less);

        // let cmp_result = x86_64::_mm_cmpeq_epi8(x86_64::__m128i::from(offsets), mask);

        // let c = simd::u8x16::from(cmp_result);
        println!("{:?}", offsets.as_array());
        // println!("{:?}", c.as_array());

        i += 16;
    }

    sum
    // (offsets, sum)
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    let arr_ptr = v.as_mut_ptr();

    const BLOCK_SIZE: usize = 128;

    let mut sum_offsets = 0;

    unsafe {
        let mut i = 0;
        while i < len - BLOCK_SIZE {
            let sum =
                collect_offsets_n::<BLOCK_SIZE, T, F>(&v[i..(i + BLOCK_SIZE)], pivot, is_less);

            // for offset in offsets {
            //     sum_offsets += (offset != OFFSET_SENTINEL) as usize;
            // }
            sum_offsets += sum;

            i += BLOCK_SIZE;
        }
    }

    sum_offsets
}

const MAX_SWAP_PAIRS: usize = 12;

#[derive(Copy, Clone)]
struct Layer {
    swap_pairs: [(u8, u8); MAX_SWAP_PAIRS],
    count: u8,
}

impl Layer {
    const fn new<const N: usize>(swap_pairs: [(u8, u8); N]) -> Self {
        assert!(N <= MAX_SWAP_PAIRS);

        let mut this_swap_pairs = [(0u8, 0u8); MAX_SWAP_PAIRS];
        unsafe {
            ptr::copy_nonoverlapping(swap_pairs.as_ptr(), this_swap_pairs.as_mut_ptr(), N);
        }

        Self {
            swap_pairs: this_swap_pairs,
            count: N as u8,
        }
    }
}

const MAX_LAYERS: usize = 16;

#[derive(Copy, Clone)]
struct Network {
    layers: [Layer; MAX_LAYERS],
    count: u8,
}

impl Network {
    const fn new<const N: usize>(layers: [Layer; N]) -> Self {
        assert!(N <= MAX_LAYERS);

        let mut this_layers = [Layer::new([]); MAX_LAYERS];
        unsafe {
            ptr::copy_nonoverlapping(layers.as_ptr(), this_layers.as_mut_ptr(), N);
        }

        Self {
            layers: this_layers,
            count: N as u8,
        }
    }
}

// This can be re-used across types.
#[rustfmt::skip]
static SORT_NETWORKS: [Network; 3] = [
    Network::new([Layer::new([(0, 2),(1, 3),(4, 6),(5, 7)]),Layer::new([(0, 4),(1, 5),(2, 6),(3, 7)]),Layer::new([(0, 1),(2, 3),(4, 5),(6, 7)]),Layer::new([(2, 4),(3, 5)]),Layer::new([(1, 4),(3, 6)]),Layer::new([(1, 2),(3, 4),(5, 6)])]),
    Network::new([Layer::new([(0, 13),(1, 12),(2, 15),(3, 14),(4, 8),(5, 6),(7, 11),(9, 10)]),Layer::new([(0, 5),(1, 7),(2, 9),(3, 4),(6, 13),(8, 14),(10, 15),(11, 12)]),Layer::new([(0, 1),(2, 3),(4, 5),(6, 8),(7, 9),(10, 11),(12, 13),(14, 15)]),Layer::new([(0, 2),(1, 3),(4, 10),(5, 11),(6, 7),(8, 9),(12, 14),(13, 15)]),Layer::new([(1, 2),(3, 12),(4, 6),(5, 7),(8, 10),(9, 11),(13, 14)]),Layer::new([(1, 4),(2, 6),(5, 8),(7, 10),(9, 13),(11, 14)]),Layer::new([(2, 4),(3, 6),(9, 12),(11, 13)]),Layer::new([(3, 5),(6, 8),(7, 9),(10, 12)]),Layer::new([(3, 4),(5, 6),(7, 8),(9, 10),(11, 12)]),Layer::new([(6, 7),(8, 9)])]),
    Network::new([Layer::new([(0, 1),(2, 3),(4, 5),(6, 7),(8, 9),(10, 11),(12, 13),(14, 15),(16, 17),(18, 19),(20, 21),(22, 23)]),Layer::new([(0, 2),(1, 3),(4, 6),(5, 7),(8, 10),(9, 11),(12, 14),(13, 15),(16, 18),(17, 19),(20, 22),(21, 23)]),Layer::new([(0, 4),(1, 5),(2, 6),(3, 7),(8, 12),(9, 13),(10, 14),(11, 15),(16, 20),(17, 21),(18, 22),(19, 23)]),Layer::new([(0, 16),(1, 18),(2, 17),(3, 19),(4, 20),(5, 22),(6, 21),(7, 23),(9, 10),(13, 14)]),Layer::new([(2, 10),(3, 11),(5, 18),(6, 14),(7, 15),(8, 16),(9, 17),(12, 20),(13, 21)]),Layer::new([(0, 8),(1, 9),(2, 12),(3, 20),(4, 16),(5, 13),(6, 17),(7, 19),(10, 18),(11, 21),(14, 22),(15, 23)]),Layer::new([(1, 8),(3, 16),(4, 12),(5, 10),(6, 9),(7, 20),(11, 19),(13, 18),(14, 17),(15, 22)]),Layer::new([(2, 4),(3, 5),(7, 13),(9, 12),(10, 16),(11, 14),(18, 20),(19, 21)]),Layer::new([(1, 2),(4, 8),(5, 9),(6, 10),(7, 11),(12, 16),(13, 17),(14, 18),(15, 19),(21, 22)]),Layer::new([(2, 4),(3, 8),(5, 6),(7, 9),(10, 12),(11, 13),(14, 16),(15, 20),(17, 18),(19, 21)]),Layer::new([(3, 5),(6, 8),(7, 10),(9, 12),(11, 14),(13, 16),(15, 17),(18, 20)]),Layer::new([(3, 4),(5, 6),(7, 8),(9, 10),(11, 12),(13, 14),(15, 16),(17, 18),(19, 20)])]),

];

#[inline(never)]
unsafe fn eval_sort_network<T, F>(network: &Network, v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    fn eval_swap_pair<T, F>(layer: &Layer, offset: usize, arr_ptr: *mut T, is_less: &mut F)
    where
        F: FnMut(&T, &T) -> bool,
    {
        // SAFETY: the network must match the input size.
        unsafe {
            swap_if_less(
                arr_ptr,
                layer.swap_pairs[offset].0 as usize,
                layer.swap_pairs[offset].1 as usize,
                is_less,
            );
        }
    }

    let swap_pair_eval_unrolled: [fn(&Layer, *mut T, &mut F); MAX_SWAP_PAIRS] = [
        |layer: &Layer, arr_ptr: *mut T, is_less: &mut F| {
            eval_swap_pair(layer, 0, arr_ptr, is_less);
        },
        |layer: &Layer, arr_ptr: *mut T, is_less: &mut F| {
            eval_swap_pair(layer, 0, arr_ptr, is_less);
            eval_swap_pair(layer, 1, arr_ptr, is_less);
        },
        |layer: &Layer, arr_ptr: *mut T, is_less: &mut F| {
            eval_swap_pair(layer, 0, arr_ptr, is_less);
            eval_swap_pair(layer, 1, arr_ptr, is_less);
            eval_swap_pair(layer, 2, arr_ptr, is_less);
        },
        |layer: &Layer, arr_ptr: *mut T, is_less: &mut F| {
            eval_swap_pair(layer, 0, arr_ptr, is_less);
            eval_swap_pair(layer, 1, arr_ptr, is_less);
            eval_swap_pair(layer, 2, arr_ptr, is_less);
            eval_swap_pair(layer, 3, arr_ptr, is_less);
        },
        |layer: &Layer, arr_ptr: *mut T, is_less: &mut F| {
            eval_swap_pair(layer, 0, arr_ptr, is_less);
            eval_swap_pair(layer, 1, arr_ptr, is_less);
            eval_swap_pair(layer, 2, arr_ptr, is_less);
            eval_swap_pair(layer, 3, arr_ptr, is_less);
            eval_swap_pair(layer, 4, arr_ptr, is_less);
        },
        |layer: &Layer, arr_ptr: *mut T, is_less: &mut F| {
            eval_swap_pair(layer, 0, arr_ptr, is_less);
            eval_swap_pair(layer, 1, arr_ptr, is_less);
            eval_swap_pair(layer, 2, arr_ptr, is_less);
            eval_swap_pair(layer, 3, arr_ptr, is_less);
            eval_swap_pair(layer, 4, arr_ptr, is_less);
            eval_swap_pair(layer, 5, arr_ptr, is_less);
        },
        |layer: &Layer, arr_ptr: *mut T, is_less: &mut F| {
            eval_swap_pair(layer, 0, arr_ptr, is_less);
            eval_swap_pair(layer, 1, arr_ptr, is_less);
            eval_swap_pair(layer, 2, arr_ptr, is_less);
            eval_swap_pair(layer, 3, arr_ptr, is_less);
            eval_swap_pair(layer, 4, arr_ptr, is_less);
            eval_swap_pair(layer, 5, arr_ptr, is_less);
            eval_swap_pair(layer, 6, arr_ptr, is_less);
        },
        |layer: &Layer, arr_ptr: *mut T, is_less: &mut F| {
            eval_swap_pair(layer, 0, arr_ptr, is_less);
            eval_swap_pair(layer, 1, arr_ptr, is_less);
            eval_swap_pair(layer, 2, arr_ptr, is_less);
            eval_swap_pair(layer, 3, arr_ptr, is_less);
            eval_swap_pair(layer, 4, arr_ptr, is_less);
            eval_swap_pair(layer, 5, arr_ptr, is_less);
            eval_swap_pair(layer, 6, arr_ptr, is_less);
            eval_swap_pair(layer, 7, arr_ptr, is_less);
        },
        |layer: &Layer, arr_ptr: *mut T, is_less: &mut F| {
            eval_swap_pair(layer, 0, arr_ptr, is_less);
            eval_swap_pair(layer, 1, arr_ptr, is_less);
            eval_swap_pair(layer, 2, arr_ptr, is_less);
            eval_swap_pair(layer, 3, arr_ptr, is_less);
            eval_swap_pair(layer, 4, arr_ptr, is_less);
            eval_swap_pair(layer, 5, arr_ptr, is_less);
            eval_swap_pair(layer, 6, arr_ptr, is_less);
            eval_swap_pair(layer, 7, arr_ptr, is_less);
            eval_swap_pair(layer, 8, arr_ptr, is_less);
        },
        |layer: &Layer, arr_ptr: *mut T, is_less: &mut F| {
            eval_swap_pair(layer, 0, arr_ptr, is_less);
            eval_swap_pair(layer, 1, arr_ptr, is_less);
            eval_swap_pair(layer, 2, arr_ptr, is_less);
            eval_swap_pair(layer, 3, arr_ptr, is_less);
            eval_swap_pair(layer, 4, arr_ptr, is_less);
            eval_swap_pair(layer, 5, arr_ptr, is_less);
            eval_swap_pair(layer, 6, arr_ptr, is_less);
            eval_swap_pair(layer, 7, arr_ptr, is_less);
            eval_swap_pair(layer, 8, arr_ptr, is_less);
            eval_swap_pair(layer, 9, arr_ptr, is_less);
        },
        |layer: &Layer, arr_ptr: *mut T, is_less: &mut F| {
            eval_swap_pair(layer, 0, arr_ptr, is_less);
            eval_swap_pair(layer, 1, arr_ptr, is_less);
            eval_swap_pair(layer, 2, arr_ptr, is_less);
            eval_swap_pair(layer, 3, arr_ptr, is_less);
            eval_swap_pair(layer, 4, arr_ptr, is_less);
            eval_swap_pair(layer, 5, arr_ptr, is_less);
            eval_swap_pair(layer, 6, arr_ptr, is_less);
            eval_swap_pair(layer, 7, arr_ptr, is_less);
            eval_swap_pair(layer, 8, arr_ptr, is_less);
            eval_swap_pair(layer, 9, arr_ptr, is_less);
            eval_swap_pair(layer, 10, arr_ptr, is_less);
        },
        |layer: &Layer, arr_ptr: *mut T, is_less: &mut F| {
            eval_swap_pair(layer, 0, arr_ptr, is_less);
            eval_swap_pair(layer, 1, arr_ptr, is_less);
            eval_swap_pair(layer, 2, arr_ptr, is_less);
            eval_swap_pair(layer, 3, arr_ptr, is_less);
            eval_swap_pair(layer, 4, arr_ptr, is_less);
            eval_swap_pair(layer, 5, arr_ptr, is_less);
            eval_swap_pair(layer, 6, arr_ptr, is_less);
            eval_swap_pair(layer, 7, arr_ptr, is_less);
            eval_swap_pair(layer, 8, arr_ptr, is_less);
            eval_swap_pair(layer, 9, arr_ptr, is_less);
            eval_swap_pair(layer, 10, arr_ptr, is_less);
            eval_swap_pair(layer, 11, arr_ptr, is_less);
        },
    ];

    let arr_ptr = v.as_mut_ptr();

    for layer_i in 0..network.count {
        let layer = network.layers[layer_i as usize];

        swap_pair_eval_unrolled[layer.count as usize - 1](&layer, arr_ptr, is_less);
        // for swap_pair_i in 0..layer.count {
        //     unsafe {
        //         swap_if_less(
        //             arr_ptr,
        //             layer.swap_pairs[swap_pair_i as usize].0 as usize,
        //             layer.swap_pairs[swap_pair_i as usize].1 as usize,
        //             is_less,
        //         );
        //     }
        // }
    }
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
unsafe fn sort8_to_40<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: caller must ensure v.len() <= 40.
    let len = v.len();
    debug_assert!(len >= 8 && len <= 40);

    // TODO table lookup.
    if len == 8 {
        eval_sort_network(&SORT_NETWORKS[0], v, is_less);
    } else if len == 16 {
        eval_sort_network(&SORT_NETWORKS[1], v, is_less);
    } else if len == 24 {
        eval_sort_network(&SORT_NETWORKS[2], v, is_less);
    }
}

// Seems allowing unrolling is not doing this any good.
/// Find the next offset where pred yields true.
/// May call pred slightly more often than strictly necessary to allow unrolling.
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
#[inline]
fn find_elem_pred<T, F>(v: &[T], mut pred: F) -> usize
where
    F: FnMut(&T) -> bool,
{
    let len = v.len();

    const MIN_UNROLL_SIZE: usize = 256;
    if len < MIN_UNROLL_SIZE {
        return v.iter().position(pred).unwrap_or(len);
    }

    let mut offset = 0;

    // May call pred at most UNROLL_SIZE times too many times.
    const UNROLL_SIZE: usize = 4;
    let unroll_end = len - UNROLL_SIZE;

    while offset < unroll_end {
        let mut pred_matches = 0;
        for i in 0..UNROLL_SIZE {
            // SAFETY: offset + UNROLL_SIZE is < len.
            let elem: &T = unsafe { v.get_unchecked(offset + i) };
            pred_matches += !pred(elem) as usize;
        }

        if pred_matches != UNROLL_SIZE {
            break;
        }

        offset += UNROLL_SIZE;
    }

    // SAFETY: offset is less than len.
    while offset < len && !pred(unsafe { v.get_unchecked(offset) }) {
        offset += 1;
    }

    offset
}

/// Find the next offset where pred yields true, reverse scanning.
/// May call pred slightly more often than strictly necessary to allow unrolling.
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
#[inline]
fn find_elem_pred_rev<T, F>(v: &[T], mut pred: F) -> usize
where
    F: FnMut(&T) -> bool,
{
    let len = v.len();

    const MIN_UNROLL_SIZE: usize = 256;
    if len < MIN_UNROLL_SIZE {
        return v
            .iter()
            .rev()
            .position(pred)
            .map(|pos| len - pos)
            .unwrap_or(0);
    }

    let mut offset = len - 1;

    // May call pred at most UNROLL_SIZE times too many times.
    const UNROLL_SIZE: usize = 4;

    while offset >= UNROLL_SIZE {
        let mut pred_matches = 0;
        for i in 0..UNROLL_SIZE {
            // SAFETY: offset + UNROLL_SIZE is < len.
            let elem: &T = unsafe { v.get_unchecked(offset - i) };
            pred_matches += !pred(elem) as usize;
        }

        if pred_matches != UNROLL_SIZE {
            break;
        }

        offset -= UNROLL_SIZE;
    }

    // SAFETY: offset is less than len.
    while offset > 0 && !pred(unsafe { v.get_unchecked(offset) }) {
        offset -= 1;
    }

    offset
}

fn x() {
// Pivot selection stuff
median9_optimal(&mut v[len_div_2..(len_div_2 + 9)], is_less);

// SAFETY: We know len >= 50, which makes (len / 2) + 9 a valid window.
let swap_count = unsafe {
    ptr::copy_nonoverlapping(v.as_ptr().add(len_div_2), swap_ptr, 9);
    median9_optimal(&mut *ptr::slice_from_raw_parts_mut(swap_ptr, 9), is_less)
};

// The math works out such that out of the 19 comparisons 10 will be true in the median network
// in median9_optimal.
const FULL_REV_SWAPS: usize = 10;
if swap_count != FULL_REV_SWAPS {
    // dbg!(swap_count);
    return (len_div_2 + 4, swap_count == 0);
}

// Chances are it is full reversed, but we are not sure. Verify that the original slice is fully
// descending.
// SAFETY: See above reasoning about accessing that window.
unsafe {
    let mut i = 1;
    while i < 9
        && is_less(
            v.get_unchecked(len_div_2 + i),
            v.get_unchecked(len_div_2 + i - 1),
        )
    {
        i += 1;
    }
    if i == 9 {
        v.reverse();
    }

    (len_div_2 + 4, i == 9)
}

// } else {
//     median9_optimal(&mut v[0..9], is_less);
//     median9_optimal(&mut v[len_div_2..(len_div_2 + 9)], is_less);
//     median9_optimal(&mut v[(len - 9)..len], is_less);

//     // SAFETY: TODO
//     unsafe {
//         let arr_ptr = v.as_mut_ptr();
//         let a = &*arr_ptr.add(4);
//         let b = &*arr_ptr.add(len_div_2 + 4);
//         let c = &*arr_ptr.add(len - 5);

//         // ptr::swap_nonoverlapping(arr_ptr.add(0), arr_ptr.add(4), 1);
//         // ptr::swap_nonoverlapping(arr_ptr.add(1), arr_ptr.add(len_div_2 + 4), 1);
//         // ptr::swap_nonoverlapping(arr_ptr.add(2), arr_ptr.add(len - 5), 1);
//     }

//     // sort3_optimal(&mut v[0..3], is_less);

//     (1, false)
// }

// let sample_elements = idx_gather(0, 9, len / 10);
// let median_elem = median9_optimal(sample_elements, is_less);
// let idx = elem_to_offset(0, 9, len / 10, median_elem).unwrap();
// (idx, false)

// if len < 128 {
//     let sample_elements = idx_gather(0, 9, len / 10);
//     let median_elem = median9_optimal(sample_elements, is_less);
//     let idx = elem_to_offset(0, 9, len / 10, median_elem).unwrap();
//     (idx, false)
// } else if len < 512 {
//     todo!("3 * 5");
// } else {
//     // TODO special care about cachelines.
//     todo!("3 * 9");
// }

    // let arr_ptr = v.as_ptr();

// const MAX_SWAP_SIZE: usize = 16;
// // const T_SIZE: usize = mem::size_of::<T>();

// let mut swap = mem::MaybeUninit::<[T; MAX_SWAP_SIZE]>::uninit();
// let mut swap_ptr = swap.as_mut_ptr() as *mut T;

// let mut idx_gather = |start: usize, steps: usize, step_size: usize| {
//     debug_assert!(start + (steps * step_size) < len && steps <= MAX_SWAP_SIZE);
//     for i in 0..steps {
//         let idx = start + (i * step_size);
//         // SAFETY: TODO
//         unsafe {
//             ptr::copy_nonoverlapping(arr_ptr.add(idx), swap_ptr.add(i), 1);
//         }
//     }

//     unsafe { &mut *ptr::slice_from_raw_parts_mut(swap_ptr, steps) }
// };

// let elem_to_offset =
//     |start: usize, steps: usize, step_size: usize, median_elem: &T| -> Option<usize> {
//         // Keep in sync with step logic in idx_gather.
//         for i in 0..steps {
//             let idx = start + (i * step_size);
//             // SAFETY: We checked in idx_gather that this element access was safe.
//             let elem = unsafe { &*arr_ptr.add(idx) };

//             // Only used to compare identical elements, that are Copy
//             // and can thus not have changed via comparison.
//             if to_ne_bytes(elem) == to_ne_bytes(median_elem) {
//                 return Some(idx);
//             }
//         }

//         None
//     };

// // if len < 50 {
// //     let sample_elements = idx_gather(0, 3, len / 4);
// //     sort3_optimal(&mut v[0..3], is_less);
// //     return (1, false);
// // }

    // fn to_ne_bytes<T>(val: &T) -> &[u8] {
//     // SAFETY: Const byte layout of T.
//     unsafe { &*ptr::slice_from_raw_parts(val as *const T as *const u8, mem::size_of::<T>()) }
// }
}

// Branchless insertion sort
/// Inserts `v[v.len() - 1]` into pre-sorted sequence `v[..v.len() - 1]` so that whole `v[..]`
/// becomes sorted.
unsafe fn insert_tail_branchless<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    debug_assert!(v.len() >= 2);

    let arr_ptr = v.as_mut_ptr();

    let tail_elem_pos = v.len() - 1;

    // SAFETY: caller must ensure v is at least len 2.
    unsafe {
        // See insert_head which talks about why this approach is beneficial.
        let tail_elem_ptr = arr_ptr.add(tail_elem_pos);

        // It's important, that we use tmp for comparison from now on. As it is the value that
        // will be copied back. And notionally we could have created a divergence if we copy
        // back the wrong value.
        let tmp = mem::ManuallyDrop::new(ptr::read(tail_elem_ptr));
        // Intermediate state of the insertion process is always tracked by `hole`, which
        // serves two purposes:
        // 1. Protects integrity of `v` from panics in `is_less`.
        // 2. Fills the remaining hole in `v` in the end.
        //
        // Panic safety:
        //
        // If `is_less` panics at any point during the process, `hole` will get dropped and
        // fill the hole in `v` with `tmp`, thus ensuring that `v` still holds every object it
        // initially held exactly once.
        let mut hole = InsertionHole {
            src: &*tmp,
            dest: tail_elem_ptr,
        };

        for _ in 0..tail_elem_pos {
            let current_ptr = hole.dest;
            hole.dest = hole.dest.sub(1);
            let is_l = is_less(&*tmp, &*hole.dest);
            hole.dest = hole.dest.add(!is_l as usize);
            ptr::copy(hole.dest, current_ptr, 1);
        }

        // `hole` gets dropped and thus copies `tmp` into the remaining hole in `v`.
    }
}

trait CopyTypeImpl: Sized {
    unsafe fn ptr_copy(src: *const Self, dest: *mut Self);
    unsafe fn ptr_copy_nonoverlapping(src: *const Self, dest: *mut Self);
}

impl<T> CopyTypeImpl for T {
    default unsafe fn ptr_copy(src: *const Self, dest: *mut Self) {
        ptr::copy(src, dest, 1);
    }

    default unsafe fn ptr_copy_nonoverlapping(src: *const Self, dest: *mut Self) {
        ptr::copy_nonoverlapping(src, dest, 1);
    }
}

impl<T: Copy> CopyTypeImpl for T {
    default unsafe fn ptr_copy(src: *const Self, dest: *mut Self) {
        *dest = *src;
    }

    default unsafe fn ptr_copy_nonoverlapping(src: *const Self, dest: *mut Self) {
        // TODO should we emit copy_nonoverlapping for release builds?
        *dest = *src;
    }
}
