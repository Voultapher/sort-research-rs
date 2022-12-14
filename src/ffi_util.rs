#![allow(dead_code, unused_macros)] // Dependent on optional features.

use std::cmp::{Ord, Ordering, PartialOrd};
use std::ffi::c_char;
use std::ptr;
use std::str;

#[repr(C)]
pub(crate) struct CompResult {
    cmp_result: i8, // -1 == less, 0 == equal, 1 == more
    is_panic: bool,
}

#[repr(C)]
pub struct FFIString {
    data: *mut c_char,
    len: usize,
    capacity: usize,
}

impl FFIString {
    pub fn new(val: String) -> Self {
        let (data, len, capacity) = val.into_raw_parts();
        Self {
            data: data as *mut c_char,
            len,
            capacity,
        }
    }

    pub fn as_str(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(&*ptr::slice_from_raw_parts(
                self.data as *const u8,
                self.len,
            ))
        }
    }
}

impl PartialEq for FFIString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for FFIString {}

impl PartialOrd for FFIString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_str().partial_cmp(&other.as_str())
    }
}

impl Ord for FFIString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl std::fmt::Debug for FFIString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.as_str())
    }
}

impl Clone for FFIString {
    fn clone(&self) -> Self {
        Self::new(self.as_str().to_owned())
    }
}

impl Drop for FFIString {
    fn drop(&mut self) {
        let str = unsafe { String::from_raw_parts(self.data as *mut u8, self.len, self.capacity) };
        drop(str);
    }
}

// Very large stack value.
#[repr(C)]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FFIOneKiloByte {
    values: [i64; 128],
}

impl FFIOneKiloByte {
    pub fn new(val: i32) -> Self {
        let mut values = [0i64; 128];
        let mut val_i64 = val as i64;

        for elem in &mut values {
            *elem = val_i64;
            val_i64 = std::hint::black_box(val_i64 + 1);
        }
        Self { values }
    }

    fn as_i64(&self) -> i64 {
        self.values[11] + self.values[55] + self.values[77]
    }
}

impl PartialOrd for FFIOneKiloByte {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_i64().partial_cmp(&other.as_i64())
    }
}

impl Ord for FFIOneKiloByte {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

// 16 byte stack value, with more expensive comparison.
#[repr(C)]
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct F128 {
    x: f64,
    y: f64,
}

impl F128 {
    pub fn new(val: i32) -> Self {
        let val_f = (val as f64) + (i32::MAX as f64) + 10.0;

        let x = val_f + 0.1;
        let y = val_f.log(4.1);

        assert!(y < x);
        assert!(x.is_normal() && y.is_normal());

        Self { x, y }
    }
}

// This is kind of hacky, but we know we only have normal comparable floats in there.
impl Eq for F128 {}

// Goal is similar code-gen between Rust and C++
// - Rust https://godbolt.org/z/3YM3xenPP
// - C++ https://godbolt.org/z/178M6j1zz
impl PartialOrd for F128 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Simulate expensive comparison function.
        let this_div = self.x / self.y;
        let other_div = other.x / other.y;

        // SAFETY: We checked in the ctor that both are normal.
        let cmp_result = unsafe { this_div.partial_cmp(&other_div).unwrap_unchecked() };

        Some(cmp_result)
    }
}

impl Ord for F128 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
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
    (
        $name:expr,
        $sort_name_prefix:ident
    ) => {
        use std::cmp::Ordering;

        use crate::ffi_util::{rust_fn_cmp, CompResult, FFIOneKiloByte, FFIString, F128};

        sort_impl!($name);

        paste::paste! {
            extern "C" {
                fn [<$sort_name_prefix _i32>](data: *mut i32, len: usize);
                fn [<$sort_name_prefix _i32_by>](
                    data: *mut i32,
                    len: usize,
                    cmp_fn: unsafe extern "C" fn(&i32, &i32, *mut u8) -> CompResult,
                    cmp_fn_ctx: *mut u8,
                ) -> u32;
                fn [<$sort_name_prefix _u64>](data: *mut u64, len: usize);
                fn [<$sort_name_prefix _u64_by>](
                    data: *mut u64,
                    len: usize,
                    cmp_fn: unsafe extern "C" fn(&u64, &u64, *mut u8) -> CompResult,
                    cmp_fn_ctx: *mut u8,
                ) -> u32;
                fn [<$sort_name_prefix _ffi_string>](data: *mut FFIString, len: usize);
                fn [<$sort_name_prefix _ffi_string_by>](
                    data: *mut FFIString,
                    len: usize,
                    cmp_fn: unsafe extern "C" fn(&FFIString, &FFIString, *mut u8) -> CompResult,
                    cmp_fn_ctx: *mut u8,
                ) -> u32;
                fn [<$sort_name_prefix _f128>](data: *mut F128, len: usize);
                fn [<$sort_name_prefix _f128_by>](
                    data: *mut F128,
                    len: usize,
                    cmp_fn: unsafe extern "C" fn(&F128, &F128, *mut u8) -> CompResult,
                    cmp_fn_ctx: *mut u8,
                ) -> u32;
                fn [<$sort_name_prefix _1k>](data: *mut FFIOneKiloByte, len: usize);
                fn [<$sort_name_prefix _1k_by>](
                    data: *mut FFIOneKiloByte,
                    len: usize,
                    cmp_fn: unsafe extern "C" fn(&FFIOneKiloByte, &FFIOneKiloByte, *mut u8) -> CompResult,
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
                fn sort(data: &mut [Self]) {
                    unsafe {
                        [<$sort_name_prefix _i32>](data.as_mut_ptr(), data.len());
                    }
                }

                fn sort_by<F: FnMut(&Self, &Self) -> Ordering>(data: &mut [Self], compare: F) {
                    make_cpp_sort_by!([<$sort_name_prefix _i32_by>], data, compare, Self);
                }
            }

            impl CppSort for u64 {
                fn sort(data: &mut [Self]) {
                    unsafe {
                        [<$sort_name_prefix _u64>](data.as_mut_ptr(), data.len());
                    }
                }

                fn sort_by<F: FnMut(&Self, &Self) -> Ordering>(data: &mut [Self], compare: F) {
                    make_cpp_sort_by!([<$sort_name_prefix _u64_by>], data, compare, Self);
                }
            }

            impl CppSort for FFIString {
                fn sort(data: &mut [Self]) {
                    unsafe {
                        [<$sort_name_prefix _ffi_string>](data.as_mut_ptr(), data.len());
                    }
                }

                fn sort_by<F: FnMut(&Self, &Self) -> Ordering>(data: &mut [Self], compare: F) {
                    make_cpp_sort_by!([<$sort_name_prefix _ffi_string_by>], data, compare, Self);
                }
            }

            impl CppSort for F128 {
                fn sort(data: &mut [Self]) {
                    unsafe {
                        [<$sort_name_prefix _f128>](data.as_mut_ptr(), data.len());
                    }
                }

                fn sort_by<F: FnMut(&Self, &Self) -> Ordering>(data: &mut [Self], compare: F) {
                    make_cpp_sort_by!([<$sort_name_prefix _f128_by>], data, compare, Self);
                }
            }

            impl CppSort for FFIOneKiloByte {
                fn sort(data: &mut [Self]) {
                    unsafe {
                        [<$sort_name_prefix _1k>](data.as_mut_ptr(), data.len());
                    }
                }

                fn sort_by<F: FnMut(&Self, &Self) -> Ordering>(data: &mut [Self], compare: F) {
                    make_cpp_sort_by!([<$sort_name_prefix _1k_by>], data, compare, Self);
                }
            }

            pub fn sort<T: Ord>(data: &mut [T]) {
                CppSort::sort(data);
            }

            pub fn sort_by<T, F: FnMut(&T, &T) -> Ordering>(data: &mut [T], compare: F) {
                CppSort::sort_by(data, compare);
            }
        } // paste
    };
}
