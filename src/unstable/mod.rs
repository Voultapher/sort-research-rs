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

// Call ips4o sort via FFI.
#[cfg(feature = "cpp_ips4o")]
pub mod cpp_ips4o;

// Call blockquicksort sort via FFI.
#[cfg(feature = "cpp_blockquicksort")]
pub mod cpp_blockquicksort;

// Call crumsort sort via FFI.
#[cfg(feature = "c_crumsort")]
pub mod c_crumsort;

// Call stdlib std::sort sort via FFI.
#[cfg(feature = "cpp_std_sys")]
pub mod cpp_std_sys;

// Call stdlib std::sort sort via FFI.
#[cfg(feature = "cpp_std_libcxx")]
pub mod cpp_std_libcxx;

// Call stdlib std::sort sort via FFI.
#[cfg(feature = "cpp_std_gcc4_3")]
pub mod cpp_std_gcc4_3;
