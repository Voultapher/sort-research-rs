use std::cmp::Ordering;

pub(crate) unsafe extern "C" fn rust_fn_cmp<T, F: FnMut(&T, &T) -> Ordering>(
    a: &T,
    b: &T,
    ctx: *mut u8,
) -> bool {
    let compare_fn = std::mem::transmute::<*mut u8, *mut F>(ctx);

    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (*compare_fn)(a, b) == Ordering::Less
    })) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Panic during compare call: {err:?}");
            // std::process::exit(1);
            false // TODO what is the right behavior here?
        }
    }
}

macro_rules! make_libcxx_sort_by {
    ($name:ident, $data:expr, $compare:expr, $type:ty) => {
        unsafe {
            let cmp_fn_ctx =
                std::mem::transmute::<*mut F, *mut u8>(Box::into_raw(Box::new($compare)));
            $name(
                $data.as_mut_ptr(),
                $data.len(),
                rust_fn_cmp::<$type, F>,
                cmp_fn_ctx,
            );

            // drop the compare function.
            let cmp_fn_ptr = std::mem::transmute::<*mut u8, *mut F>(cmp_fn_ctx);
            let _cmp_fn_box = Box::from_raw(cmp_fn_ptr);
        }
    };
}
