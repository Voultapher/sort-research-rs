use std::hint::black_box;
use std::ptr;

#[allow(dead_code)]
#[inline(never)]
fn instantiate_sort<T: Ord>(v: &mut [T]) {
    // v.sort();
    // v.sort_unstable();

    ipnsort::sort(v);
}

#[inline(never)]
fn produce_slice<T>() -> &'static mut [T] {
    // SAFETY: This is just for compile artifact measuring to have reliable side-effects. This could
    // is not meant to be run for real.
    unsafe {
        let data = black_box(ptr::null_mut());
        let len = black_box(0);
        &mut *ptr::slice_from_raw_parts_mut(data, len)
    }
}

fn main() {
    let input_u64 = produce_slice::<u64>();
    let input_string = produce_slice::<String>();

    #[cfg(feature = "sort_inst")]
    {
        #[cfg(feature = "type_u64")]
        {
            instantiate_sort(input_u64);
        }

        #[cfg(feature = "type_string")]
        {
            instantiate_sort(input_string);
        }
    }

    black_box(input_u64); // side-effect
    black_box(input_string); // side-effect
}
