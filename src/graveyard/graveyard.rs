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
    sort_comp::new_stable_sort::sort(&mut input);

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
