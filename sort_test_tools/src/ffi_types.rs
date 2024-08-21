use std::cmp::Ordering;
use std::ffi::c_char;
use std::ptr;
use std::str;

#[repr(C)]
pub struct CompResult {
    pub cmp_result: i8, // -1 == less, 0 == equal, 1 == more
    pub is_panic: bool,
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

    pub fn as_str(&self) -> Option<&str> {
        // SAFETY: See `as_str_unchecked`.
        unsafe {
            if !self.data.is_null() {
                Some(str::from_utf8_unchecked(&*ptr::slice_from_raw_parts(
                    self.data as *const u8,
                    self.len,
                )))
            } else {
                None
            }
        }
    }

    pub unsafe fn as_str_unchecked(&self) -> &str {
        // SAFETY: The value is valid by construction so from a Rust interface perspective it is a
        // safe function. However it's possible that C++ sort implementations leave the value in a
        // moved from state. To keep benchmarks fair this is the one used for the comparison impl
        // where exception safety tests are not relevant.
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
        // SAFETY: See `as_str_unchecked`.
        unsafe { self.as_str_unchecked() == other.as_str_unchecked() }
    }
}

impl Eq for FFIString {}

impl PartialOrd for FFIString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FFIString {
    fn cmp(&self, other: &Self) -> Ordering {
        // SAFETY: See `as_str_unchecked`.
        unsafe { self.as_str_unchecked().cmp(other.as_str_unchecked()) }
    }
}

impl std::fmt::Debug for FFIString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{:?}\"", self.as_str())
    }
}

impl Clone for FFIString {
    fn clone(&self) -> Self {
        Self::new(self.as_str().unwrap().to_owned())
    }
}

impl Drop for FFIString {
    fn drop(&mut self) {
        if !self.data.is_null() {
            let str =
                unsafe { String::from_raw_parts(self.data as *mut u8, self.len, self.capacity) };
            drop(str);
        }
    }
}

// Very large stack value.
#[repr(C)]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FFIOneKibiByte {
    values: [i64; 128],
}

impl FFIOneKibiByte {
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

impl PartialOrd for FFIOneKibiByte {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FFIOneKibiByte {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_i64().cmp(&other.as_i64())
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

impl PartialOrd for F128 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// Goal is similar code-gen between Rust and C++
// - Rust https://godbolt.org/z/3YM3xenPP
// - C++ https://godbolt.org/z/178M6j1zz
impl Ord for F128 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Simulate expensive comparison function.
        let this_div = self.x / self.y;
        let other_div = other.x / other.y;

        // SAFETY: We checked in the ctor that both are normal.
        unsafe { this_div.partial_cmp(&other_div).unwrap_unchecked() }
    }
}
