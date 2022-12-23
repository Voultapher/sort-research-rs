#![allow(unused)]

use core::arch::x86_64;
use core::cmp;
use core::intrinsics;
use core::mem;
use core::ptr;

partition_impl!("avx2");

#[target_feature(enable = "avx2")]
#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
unsafe fn partition_avx2<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    todo!()
}

#[cfg_attr(feature = "no_inline_sub_functions", inline(never))]
fn partition<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // TODO feature detection.
    unsafe { partition_avx2(v, pivot, is_less) }
}
