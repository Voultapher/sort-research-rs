use std::mem;
use std::ptr;

pub fn u8_as_x<T: Sized>(data: &[u8]) -> Vec<T> {
    let data_aligned_x = &data[..(data.len() - (data.len() % mem::size_of::<T>()))];
    if data_aligned_x.is_empty() {
        return Vec::new();
    }

    let mut v: Vec<T> = Vec::with_capacity(data_aligned_x.len() / mem::size_of::<T>());
    unsafe {
        ptr::copy_nonoverlapping(
            data_aligned_x.as_ptr(),
            v.as_mut_ptr() as *mut u8,
            data_aligned_x.len(),
        );
    }

    v
}
