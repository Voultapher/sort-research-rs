pub mod rust_new;
pub mod rust_std;

#[cfg(feature = "wpwoodjr")]
pub mod rust_wpwoodjr;

// Call stdlib std::sort_stable sort via FFI.
#[cfg(feature = "cpp_std_sys")]
pub mod cpp_std_sys;

// Call stdlib std::sort_stable sort via FFI.
#[cfg(feature = "cpp_std_libcxx")]
pub mod cpp_std_libcxx;
