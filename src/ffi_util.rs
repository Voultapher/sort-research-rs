#![allow(dead_code, unused_macros)] // Dependent on optional features.

use std::cmp::Ordering;

#[repr(C)]
pub(crate) struct CompResult {
    cmp_result: i8, // -1 == less, 0 == equal, 1 == more
    is_panic: bool,
}

pub(crate) unsafe extern "C" fn rust_fn_cmp<T, F: FnMut(&T, &T) -> Ordering>(
    a: &T,
    b: &T,
    ctx: *mut u8,
) -> CompResult {
    let compare_fn = std::mem::transmute::<*mut u8, *mut F>(ctx);

    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (*compare_fn)(a, b))) {
        Ok(val) => CompResult {
            cmp_result: match val {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            },
            is_panic: false,
        },
        Err(err) => {
            eprintln!("Panic during compare call: {err:?}");
            CompResult {
                cmp_result: 0,
                is_panic: true,
            }
        }
    }
}

macro_rules! make_cpp_sort_by {
    ($name:ident, $data:expr, $compare:expr, $type:ty) => {
        unsafe {
            let cmp_fn_ctx =
                std::mem::transmute::<*mut F, *mut u8>(Box::into_raw(Box::new($compare)));
            let ret_code = $name(
                $data.as_mut_ptr(),
                $data.len(),
                rust_fn_cmp::<$type, F>,
                cmp_fn_ctx,
            );

            // drop the compare function.
            let cmp_fn_ptr = std::mem::transmute::<*mut u8, *mut F>(cmp_fn_ctx);
            let _cmp_fn_box = Box::from_raw(cmp_fn_ptr);

            if ret_code != 0 {
                panic!("Panic in comparison function");
            }
        }
    };
}

macro_rules! ffi_sort_impl {
    ($sort_i32_name:ident, $sort_i32_by_name:ident, $sort_u64_name:ident, $sort_u64_by_name:ident) => {
        use std::cmp::Ordering;

        use crate::ffi_util::{rust_fn_cmp, CompResult};

        extern "C" {
            fn $sort_i32_name(data: *mut i32, len: usize);
            fn $sort_i32_by_name(
                data: *mut i32,
                len: usize,
                cmp_fn: unsafe extern "C" fn(&i32, &i32, *mut u8) -> CompResult,
                cmp_fn_ctx: *mut u8,
            ) -> u32;
            fn $sort_u64_name(data: *mut u64, len: usize);
            fn $sort_u64_by_name(
                data: *mut u64,
                len: usize,
                cmp_fn: unsafe extern "C" fn(&u64, &u64, *mut u8) -> CompResult,
                cmp_fn_ctx: *mut u8,
            ) -> u32;
        }

        trait CppSort: Sized {
            fn sort(data: &mut [Self]);
            fn sort_by<F: FnMut(&Self, &Self) -> Ordering>(data: &mut [Self], compare: F);
        }

        impl<T> CppSort for T {
            default fn sort(_data: &mut [T]) {
                panic!("Type not supported");
            }

            default fn sort_by<F: FnMut(&T, &T) -> Ordering>(_data: &mut [T], _compare: F) {
                panic!("Type not supported");
            }
        }

        impl CppSort for i32 {
            fn sort(data: &mut [i32]) {
                unsafe {
                    $sort_i32_name(data.as_mut_ptr(), data.len());
                }
            }

            fn sort_by<F: FnMut(&i32, &i32) -> Ordering>(data: &mut [i32], compare: F) {
                make_cpp_sort_by!($sort_i32_by_name, data, compare, i32);
            }
        }

        impl CppSort for u64 {
            fn sort(data: &mut [u64]) {
                unsafe {
                    $sort_u64_name(data.as_mut_ptr(), data.len());
                }
            }

            fn sort_by<F: FnMut(&u64, &u64) -> Ordering>(data: &mut [u64], compare: F) {
                make_cpp_sort_by!($sort_u64_by_name, data, compare, u64);
            }
        }

        pub fn sort<T: Ord>(data: &mut [T]) {
            CppSort::sort(data);
        }

        pub fn sort_by<T, F: FnMut(&T, &T) -> Ordering>(data: &mut [T], compare: F) {
            CppSort::sort_by(data, compare);
        }
    };
}
