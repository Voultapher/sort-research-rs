use std::cmp::Ordering;

use crate::ffi_util::{rust_fn_cmp, CompResult};

extern "C" {
    fn fluxsort_stable_i32(data: *mut i32, len: usize);
    #[must_use]
    fn fluxsort_stable_i32_by(
        data: *mut i32,
        len: usize,
        cmp_fn: unsafe extern "C" fn(&i32, &i32, *mut u8) -> CompResult,
        cmp_fn_ctx: *mut u8,
    ) -> u32;

    fn fluxsort_stable_u64(data: *mut u64, len: usize);
    #[must_use]
    fn fluxsort_stable_u64_by(
        data: *mut u64,
        len: usize,
        cmp_fn: unsafe extern "C" fn(&u64, &u64, *mut u8) -> CompResult,
        cmp_fn_ctx: *mut u8,
    ) -> u32;
}

trait FluxSort: Sized {
    fn sort(data: &mut [Self]);
    fn sort_by<F: FnMut(&Self, &Self) -> Ordering>(data: &mut [Self], compare: F);
}

impl<T> FluxSort for T {
    default fn sort(_data: &mut [T]) {
        panic!("Type not supported");
    }

    default fn sort_by<F: FnMut(&T, &T) -> Ordering>(_data: &mut [T], _compare: F) {
        panic!("Type not supported");
    }
}

impl FluxSort for i32 {
    fn sort(data: &mut [i32]) {
        unsafe {
            fluxsort_stable_i32(data.as_mut_ptr(), data.len());
        }
    }

    fn sort_by<F: FnMut(&i32, &i32) -> Ordering>(data: &mut [i32], compare: F) {
        make_cpp_sort_by!(fluxsort_stable_i32_by, data, compare, i32);
    }
}

impl FluxSort for u64 {
    fn sort(data: &mut [u64]) {
        unsafe {
            fluxsort_stable_u64(data.as_mut_ptr(), data.len());
        }
    }

    fn sort_by<F: FnMut(&u64, &u64) -> Ordering>(data: &mut [u64], compare: F) {
        make_cpp_sort_by!(fluxsort_stable_u64_by, data, compare, u64);
    }
}

pub fn sort<T: Ord>(data: &mut [T]) {
    FluxSort::sort(data);
}

pub fn sort_by<T, F: FnMut(&T, &T) -> Ordering>(data: &mut [T], compare: F) {
    FluxSort::sort_by(data, compare);
}
