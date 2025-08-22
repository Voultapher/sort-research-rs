/// Basic TimSort without galloping mode. Branchy merge function and no insertion sort.
use std::cmp::Ordering;
use std::mem::SizedTypeProperties;
use std::ptr;

sort_impl!("timsort_evo0_stable");

#[inline]
pub fn sort<T>(v: &mut [T])
where
    T: Ord,
{
    stable_sort(v, |a, b| a.cmp(b));
}

#[inline]
pub fn sort_by<T, F>(v: &mut [T], compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    stable_sort(v, compare);
}

////////////////////////////////////////////////////////////////////////////////
// Sorting
////////////////////////////////////////////////////////////////////////////////

#[inline]
fn stable_sort<T, F>(v: &mut [T], mut compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    if T::IS_ZST {
        // Sorting has no meaningful behavior on zero-sized types. Do nothing.
        return;
    }

    merge_sort(v, &mut |a, b| compare(a, b) == Ordering::Less);
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
pub fn merge_sort<T, CmpF>(v: &mut [T], is_less: &mut CmpF)
where
    CmpF: FnMut(&T, &T) -> bool,
{
    // The caller should have already checked that.
    debug_assert!(!T::IS_ZST);

    let len = v.len();

    if len < 2 {
        // These inputs are always sorted.
        return;
    }

    // I'd argue most system are not as memory constrained that the double
    let mut buf = Vec::with_capacity(len / 2);
    let buf_ptr = buf.as_mut_ptr();

    let mut runs = Vec::new();

    let mut end = 0;
    let mut start = 0;

    // Scan forward. Memory pre-fetching prefers forward scanning vs backwards scanning, and the
    // code-gen is usually better. For the most sensitive types such as integers, these are merged
    // bidirectionally at once. So there is no benefit in scanning backwards.
    while end < len {
        let (streak_end, was_reversed) = find_streak(&v[start..], is_less);
        end += streak_end;
        if was_reversed {
            v[start..end].reverse();
        }

        // Push this run onto the stack.
        runs.push(TimSortRun {
            start,
            len: end - start,
        });
        start = end;

        // Merge some pairs of adjacent runs to satisfy the invariants.
        while let Some(r) = collapse(runs.as_slice(), len) {
            let left = runs[r];
            let right = runs[r + 1];
            let merge_slice = &mut v[left.start..right.start + right.len];
            unsafe {
                merge(merge_slice, left.len, buf_ptr, is_less);
            }
            runs[r + 1] = TimSortRun {
                start: left.start,
                len: left.len + right.len,
            };
            runs.remove(r);
        }
    }

    // Finally, exactly one run must remain in the stack.
    debug_assert!(runs.len() == 1 && runs[0].start == 0 && runs[0].len == len);

    // Examines the stack of runs and identifies the next pair of runs to merge. More specifically,
    // if `Some(r)` is returned, that means `runs[r]` and `runs[r + 1]` must be merged next. If the
    // algorithm should continue building a new run instead, `None` is returned.
    //
    // TimSort is infamous for its buggy implementations, as described here:
    // http://envisage-project.eu/timsort-specification-and-verification/
    //
    // The gist of the story is: we must enforce the invariants on the top four runs on the stack.
    // Enforcing them on just top three is not sufficient to ensure that the invariants will still
    // hold for *all* runs in the stack.
    //
    // This function correctly checks invariants for the top four runs. Additionally, if the top
    // run ends at stop, it will always demand a merge operation until the stack is fully
    // collapsed, in order to complete the sort.
    #[inline]
    fn collapse(runs: &[TimSortRun], stop: usize) -> Option<usize> {
        let n = runs.len();
        if n >= 2
            && (runs[n - 1].start + runs[n - 1].len == stop
                || runs[n - 2].len <= runs[n - 1].len
                || (n >= 3 && runs[n - 3].len <= runs[n - 2].len + runs[n - 1].len)
                || (n >= 4 && runs[n - 4].len <= runs[n - 3].len + runs[n - 2].len))
        {
            if n >= 3 && runs[n - 3].len < runs[n - 1].len {
                Some(n - 3)
            } else {
                Some(n - 2)
            }
        } else {
            None
        }
    }
}

/// Internal type used by merge_sort.
#[derive(Clone, Copy, Debug)]
pub struct TimSortRun {
    len: usize,
    start: usize,
}

/// Finds a streak of presorted elements starting at the beginning of the slice. Returns the first
/// value that is not part of said streak, and a bool denoting wether the streak was reversed.
/// Streaks can be increasing or decreasing.
fn find_streak<T, F>(v: &[T], is_less: &mut F) -> (usize, bool)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    if len < 2 {
        return (len, false);
    }

    let mut end = 2;

    // SAFETY: See below specific.
    unsafe {
        // SAFETY: We checked that len >= 2, so 0 and 1 are valid indices.
        let assume_reverse = is_less(v.get_unchecked(1), v.get_unchecked(0));

        // SAFETY: We know end >= 2 and check end < len.
        // From that follows that accessing v at end and end - 1 is safe.
        if assume_reverse {
            while end < len && is_less(v.get_unchecked(end), v.get_unchecked(end - 1)) {
                end += 1;
            }

            (end, true)
        } else {
            while end < len && !is_less(v.get_unchecked(end), v.get_unchecked(end - 1)) {
                end += 1;
            }
            (end, false)
        }
    }
}

/// Merges non-decreasing runs `v[..mid]` and `v[mid..]` using `buf` as temporary storage, and
/// stores the result into `v[..]`.
///
/// # Safety
///
/// The two slices must be non-empty and `mid` must be in bounds. Buffer `buf` must be long enough
/// to hold a copy of the shorter slice. Also, `T` must not be a zero-sized type.
///
/// Never inline this function to avoid code bloat. It still optimizes nicely and has practically no
/// performance impact.
#[inline(never)]
unsafe fn merge<T, F>(v: &mut [T], mid: usize, buf: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    assert!(mid > 0 && mid < len);

    let arr_ptr = v.as_mut_ptr();
    let (v_mid, v_end) = unsafe { (arr_ptr.add(mid), arr_ptr.add(len)) };

    // The merge process first copies the shorter run into `buf`. Then it traces the newly copied
    // run and the longer run forwards (or backwards), comparing their next unconsumed elements and
    // copying the lesser (or greater) one into `v`.
    //
    // As soon as the shorter run is fully consumed, the process is done. If the longer run gets
    // consumed first, then we must copy whatever is left of the shorter run into the remaining
    // hole in `v`.
    //
    // Intermediate state of the process is always tracked by `hole`, which serves two purposes:
    // 1. Protects integrity of `v` from panics in `is_less`.
    // 2. Fills the remaining hole in `v` if the longer run gets consumed first.
    //
    // Panic safety:
    //
    // If `is_less` panics at any point during the process, `hole` will get dropped and fill the
    // hole in `v` with the unconsumed range in `buf`, thus ensuring that `v` still holds every
    // object it initially held exactly once.
    let mut hole;

    if mid <= len - mid {
        // The left run is shorter.
        unsafe {
            ptr::copy_nonoverlapping(arr_ptr, buf, mid);
            hole = MergeHole {
                start: buf,
                end: buf.add(mid),
                dest: arr_ptr,
            };
        }

        // Initially, these pointers point to the beginnings of their arrays.
        let left = &mut hole.start;
        let mut right = v_mid;
        let out = &mut hole.dest;

        while *left < hole.end && right < v_end {
            // Consume the lesser side.
            // If equal, prefer the left run to maintain stability.
            unsafe {
                if is_less(&*right, &**left) {
                    ptr::copy_nonoverlapping(right, *out, 1);
                    right = right.add(1);
                } else {
                    ptr::copy_nonoverlapping(*left, *out, 1);
                    *left = left.add(1);
                }
                *out = out.add(1);
            }
        }
    } else {
        // The right run is shorter.
        unsafe {
            ptr::copy_nonoverlapping(v_mid, buf, len - mid);
            hole = MergeHole {
                start: buf,
                end: buf.add(len - mid),
                dest: v_mid,
            };
        }

        // Initially, these pointers point past the ends of their arrays.
        let left = &mut hole.dest;
        let right = &mut hole.end;
        let mut out = v_end;

        while arr_ptr < *left && buf < *right {
            // Consume the greater side.
            // If equal, prefer the right run to maintain stability.

            // LLVM is actually pretty at turning this into cmov based code.
            // For demonstration purposes I want this to generate a cmp + jmp.
            unsafe {
                if is_less(&*right.offset(-1), &*left.offset(-1)) {
                    out = out.sub(1);
                    *left = left.sub(1);
                    ptr::copy_nonoverlapping(*left, out, 1);

                    // Stupid check that will always add 0 to out. But it tricks LLVM into
                    // generating the jmp instead of cmov for the main comparison.
                    out = out.add((out > v_end) as usize);
                } else {
                    out = out.sub(1);
                    *right = right.sub(1);
                    ptr::copy_nonoverlapping(*right, out, 1);
                }
            }
        }
    }
    // Finally, `hole` gets dropped. If the shorter run was not fully consumed, whatever remains of
    // it will now be copied into the hole in `v`.

    // When dropped, copies the range `start..end` into `dest..`.
    struct MergeHole<T> {
        start: *mut T,
        end: *mut T,
        dest: *mut T,
    }

    impl<T> Drop for MergeHole<T> {
        fn drop(&mut self) {
            // `T` is not a zero-sized type, and these are pointers into a slice's elements.
            unsafe {
                let len = self.end.offset_from_unsigned(self.start);
                ptr::copy_nonoverlapping(self.start, self.dest, len);
            }
        }
    }
}
