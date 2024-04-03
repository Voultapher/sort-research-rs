pub mod rust_ipnsort;
pub mod rust_std;

#[cfg(feature = "rust_dmsort")]
pub mod rust_dmsort;

#[cfg(feature = "rust_crumsort_rs")]
pub mod rust_crumsort_rs;

#[cfg(feature = "rust_tinysort")]
pub mod rust_tinysort;

// Call pdqsort sort via FFI.
#[cfg(feature = "cpp_pdqsort")]
pub mod cpp_pdqsort;

// Call ips4o sort via FFI.
#[cfg(feature = "cpp_ips4o")]
pub mod cpp_ips4o;

// Call blockquicksort sort via FFI.
#[cfg(feature = "cpp_blockquicksort")]
pub mod cpp_blockquicksort;

// Call gerbens quicksort sort via FFI.
#[cfg(feature = "cpp_gerbens_qsort")]
pub mod cpp_gerbens_qsort;

// Call nanosort via FFI.
#[cfg(feature = "cpp_nanosort")]
pub mod cpp_nanosort;

// Call qsort sort via FFI.
#[cfg(feature = "c_std_sys")]
pub mod c_std_sys;

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

// Call golang slices.Sort
#[cfg(feature = "golang_std")]
pub mod golang_std;
