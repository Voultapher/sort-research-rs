use std::cmp::Ordering;
use std::ptr;
use std::sync::Mutex;

use crumsort;

sort_impl!("rust_crumsort_rs_unstable");

trait Crumsort: Sized {
    fn sort(data: &mut [Self]);
}

impl<T> Crumsort for T {
    default fn sort(_data: &mut [Self]) {
        panic!("Type not supported.");
    }
}

impl<T: Copy + Default + Send + Ord> Crumsort for T {
    fn sort(data: &mut [Self]) {
        crumsort::ParCrumSort::par_crumsort(data);
    }
}

struct OrdWrapper<T> {
    val: *const T,
    compare_fn: fn(*const T, *const T, *const u8) -> Ordering,
    ctx: *const u8,
}

impl<T> OrdWrapper<T> {
    fn ord_val(&self, other: &Self) -> Ordering {
        (self.compare_fn)(self.val, other.val, self.ctx)
    }
}

impl<T> PartialEq for OrdWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ord_val(other).is_eq()
    }
}

impl<T> Eq for OrdWrapper<T> {}

impl<T> PartialOrd for OrdWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.ord_val(other))
    }
}

impl<T> Ord for OrdWrapper<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<T> Default for OrdWrapper<T> {
    fn default() -> Self {
        Self {
            val: std::ptr::null(),
            compare_fn: |_a, _b, _ctx| panic!("Called compare on default value."),
            ctx: std::ptr::null(),
        }
    }
}

// SAFETY: The context pointer is a pointer to a Mutex, that should be Send.
unsafe impl<T> Send for OrdWrapper<T> {}

impl<T> Clone for OrdWrapper<T> {
    fn clone(&self) -> Self {
        unreachable!()
    }
}

impl<T> Copy for OrdWrapper<T> {}

pub fn sort<T: Ord>(data: &mut [T]) {
    <T as Crumsort>::sort(data);
}

pub fn sort_by<T, F: FnMut(&T, &T) -> Ordering>(data: &mut [T], compare: F) {
    let compare_fn: fn(a_ptr: *const T, b_ptr: *const T, *const u8) -> Ordering =
        |a_ptr, b_ptr, ctx| {
            // The caller MUST ensure that both pointers are valid.
            let a = unsafe { &*a_ptr };
            let b = unsafe { &*b_ptr };

            // The caller MUST ensure that ctx lives long enough as has the correct layout.
            let impl_fn = unsafe { &*(ctx as *const Mutex<F>) };
            (impl_fn.lock().unwrap())(a, b)
        };

    let ctx: Box<Mutex<F>> = Box::new(Mutex::new(compare));
    let ctx_ptr = ctx.as_ref() as *const Mutex<F> as *const u8;

    // This simulates having an Ord implementation. There a temporary wrapper Vec wouldn't exist.
    // That also makes the copy back regardless of panic sensible. All of this is very inefficient
    // but that's not the point, this will not be used for benchmarking runtime.
    let mut wrapped_data = data
        .iter()
        .map(|val| OrdWrapper::<T> {
            val,
            compare_fn,
            ctx: ctx_ptr,
        })
        .collect::<Vec<OrdWrapper<T>>>();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        crumsort::ParCrumSort::par_crumsort(wrapped_data.as_mut_slice());
    }));

    let len = data.len();
    let mut tmp_array: Vec<T> = Vec::with_capacity(data.len());
    let tmp_array_ptr = tmp_array.as_mut_ptr();

    for i in 0..len {
        // SAFETY: The pointer must be valid as created by us and not a Default element.
        unsafe {
            ptr::copy_nonoverlapping(wrapped_data[i].val, tmp_array_ptr.add(i), 1);
        }
    }

    // SAFETY: tmp_array was initialized correctly and contains no duplicates. We essentially
    // overwrite the value with themselves. No new owners where created in the meantime. This is
    // more akin to move or mem::take than performing a Copy or Clone.
    unsafe {
        ptr::copy_nonoverlapping(tmp_array_ptr, data.as_mut_ptr(), len);
    }

    if result.is_err() {
        panic!("{:?}", result.err());
    }
}
