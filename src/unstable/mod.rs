pub mod rust_new;
pub mod rust_std;

#[cfg(feature = "emilk_dmsort")]
pub mod rust_dmsort;

// Call pdqsort sort via FFI.
#[cfg(feature = "cpp_pdqsort")]
pub mod cpp_pdqsort;

// Call crumsort sort via FFI.
#[cfg(feature = "c_crumsort")]
pub mod c_crumsort;

// Call fluxsort sort via FFI.
// While this sort claims to be stable, testing shows it isn't.
#[cfg(feature = "c_fluxsort")]
pub mod c_fluxsort;

// Call stdlib std::sort sort via FFI.
#[cfg(feature = "cpp_std_sys")]
pub mod cpp_std_sys;

// Call stdlib std::sort sort via FFI.
#[cfg(feature = "cpp_std_libcxx")]
pub mod cpp_std_libcxx;
