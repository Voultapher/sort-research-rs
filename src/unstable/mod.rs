pub mod rust_new;
pub mod rust_std;

#[cfg(feature = "rust_dmsort")]
pub mod rust_dmsort;

// Call pdqsort sort via FFI.
#[cfg(feature = "cpp_pdqsort")]
pub mod cpp_pdqsort;

// Call simdsort sort via FFI.
#[cfg(feature = "cpp_simdsort")]
pub mod cpp_simdsort;

// Call crumsort sort via FFI.
#[cfg(feature = "c_crumsort")]
pub mod c_crumsort;

// Call stdlib std::sort sort via FFI.
#[cfg(feature = "cpp_std_sys")]
pub mod cpp_std_sys;

// Call stdlib std::sort sort via FFI.
#[cfg(feature = "cpp_std_libcxx")]
pub mod cpp_std_libcxx;
