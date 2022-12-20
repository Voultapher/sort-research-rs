use core::mem;

// version found in crumsort-rs

partition_impl!("crumsort");

/// Size of tack-allocated swap memory for certain operations that can be performed faster with
/// swap memory than in-place.
const SWAP_SIZE: usize = 512;
/// Size of likely unrolled loops when performing partitioning that does not fit in swap.
const CRUM_CHUNK_SIZE: usize = 32;
/// Partition ratio at which point quadsort is used.
// const CRUM_OUT: usize = 24;

#[derive(Debug)]
struct Partitioner {
    left_i: usize,
    right_i: usize,
    end_i: usize,
    cursor: usize,
}

pub(crate) trait Sortable: Copy + Default + Ord {}

impl<T: Copy + Default + Ord> Sortable for T {}

impl Partitioner {
    pub const fn new<const C: usize>(len: usize) -> Self {
        Self {
            left_i: C,
            right_i: len - 1 - C,
            end_i: len - 1,
            cursor: 0,
        }
    }

    pub fn next<T: Sortable, const IS_LEFT: bool>(
        &mut self,
        slice: &mut [T],
        swap: Option<&mut [T]>,
        pivot: T,
    ) {
        let val = {
            let i = if IS_LEFT { self.left_i } else { self.right_i };
            let val = swap.map_or_else(
                || {
                    // SAFETY:
                    // `Partitioner::next` is called `slice.len()` times ==> 0 <= i < `slice.len()`
                    unsafe { *slice.get_unchecked(i) }
                },
                |swap| {
                    // SAFETY:
                    // `Partitioner::next` is called `slice.len()` times ==> 0 <= i < `slice.len()` (1)
                    // i < 2 * CRUM_CHUNK_SIZE (2)
                    // 2 * CRUM_CHUNK_SIZE <= `swap.len()` (3)
                    // (1), (2), (3) ==> 0 <= i < `swap.len()`
                    unsafe { *swap.get_unchecked(i) }
                },
            );

            // SAFETY:
            // `usize::from(val <= pivot)` <= 1, `Partitioner::next` is called `slice.len()`
            // times ==> self.cursor <= i` (1)
            // (1), i < `slice.len()` ==> self.cursor < `slice.len()`
            unsafe {
                *slice.get_unchecked_mut(self.cursor) = val;
            }

            // SAFETY:
            // for nth iteration:
            //   `usize::from(val <= pivot)` <= 1 ==> self.cursor <= n (1)
            //   self.end_i = `slice.len()` - 1 - n (2)
            //   (1), (2) ==> self.cursor + self.end_i <= `slice.len()` - 1 <==>
            //   <==> self.cursor + self.end_i < `slice.len()`
            unsafe {
                *slice.get_unchecked_mut(self.cursor + self.end_i) = val;
            }

            val
        };

        if IS_LEFT {
            self.left_i += 1;
        } else {
            self.right_i -= 1;
        }

        self.end_i = self.end_i.overflowing_sub(1).0;
        self.cursor += usize::from(val <= pivot);
    }
}

fn fulcrum_partition_inner<T: Sortable>(slice: &mut [T], swap: &mut [T], pivot: T) -> usize {
    if slice.len() <= swap.len() {
        const CHUNK: usize = 8;

        let mut i = 0;
        let mut cursor = 0;

        let mut partition = |slice: &mut [T]| {
            let val = {
                // SAFETY:
                // `partition` is called `slice.len()` times ==> i < `slice.len()`
                let val = unsafe { *slice.get_unchecked(i) };

                // SAFETY:
                // `swap.len` >= `slice.len()` ==> i < `swap.len()` (1)
                // `usize::from(val <= pivot)` <= 1, `partition` is called `slice.len()` times ==>
                // => cursor <= i (2)
                // (1), (2) ==> 0 <= i - cursor < `swap.len()`
                unsafe {
                    *swap.get_unchecked_mut(i - cursor) = val;
                }

                // SAFETY:
                // cursor <= i, i < `slice.len()` ==> cursor < `slice.len()`
                unsafe {
                    *slice.get_unchecked_mut(cursor) = val;
                }

                val
            };

            i += 1;
            cursor += usize::from(val <= pivot);
        };

        for _ in 0..slice.len() / CHUNK {
            for _ in 0..CHUNK {
                partition(slice);
            }
        }

        for _ in 0..slice.len() % CHUNK {
            partition(slice);
        }

        let len = slice.len();
        slice[cursor..].copy_from_slice(&swap[..len - cursor]);

        return cursor;
    }

    swap[..CRUM_CHUNK_SIZE].copy_from_slice(&slice[..CRUM_CHUNK_SIZE]);
    swap[CRUM_CHUNK_SIZE..2 * CRUM_CHUNK_SIZE]
        .copy_from_slice(&slice[slice.len() - CRUM_CHUNK_SIZE..]);

    let mut partitioner = Partitioner::new::<CRUM_CHUNK_SIZE>(slice.len());

    let mut count = slice.len() / CRUM_CHUNK_SIZE - 2;

    loop {
        if partitioner.left_i - partitioner.cursor <= CRUM_CHUNK_SIZE {
            if let Some(new_count) = count.checked_sub(1) {
                count = new_count;
            } else {
                break;
            }

            for _ in 0..CRUM_CHUNK_SIZE {
                partitioner.next::<_, true>(slice, None, pivot);
            }
        }

        if partitioner.left_i - partitioner.cursor > CRUM_CHUNK_SIZE {
            if let Some(new_count) = count.checked_sub(1) {
                count = new_count;
            } else {
                break;
            }

            for _ in 0..CRUM_CHUNK_SIZE {
                partitioner.next::<_, false>(slice, None, pivot);
            }
        }
    }

    if partitioner.left_i - partitioner.cursor <= CRUM_CHUNK_SIZE {
        for _ in 0..slice.len() % CRUM_CHUNK_SIZE {
            partitioner.next::<_, true>(slice, None, pivot);
        }
    } else {
        for _ in 0..slice.len() % CRUM_CHUNK_SIZE {
            partitioner.next::<_, false>(slice, None, pivot);
        }
    }

    partitioner.left_i = 0;
    for _ in 0..2 * CRUM_CHUNK_SIZE {
        partitioner.next::<_, true>(slice, Some(swap), pivot);
    }

    partitioner.cursor
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F>(v: &mut [T], pivot: &T, _is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    if mem::size_of::<T>() == 8 {
        unsafe {
            let pivot_x = mem::transmute::<&T, &u64>(pivot);
            let v_x = mem::transmute::<&mut [T], &mut [u64]>(v);

            // That's UB, but more representative of the actual crumsort implementation,
            // fulcrum_partition_inner ought to work with a pointer.
            // let mut swap = mem::MaybeUninit::<[u64; SWAP_SIZE]>::uninit();
            // return fulcrum_partition_inner(v_x, &mut swap.assume_init(), *pivot_x);

            let mut swap = [0u64; SWAP_SIZE];
            return fulcrum_partition_inner(v_x, &mut swap, *pivot_x);
        }
    }

    todo!()
}
