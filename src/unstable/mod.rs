pub mod rust_new;
pub mod rust_std;

#[cfg(feature = "emilk_dmsort")]
pub mod rust_dmsort;

// Call pdqsort sort via FFI.
#[cfg(feature = "cpp_pdqsort")]
pub mod cpp_pdqsort;

// Call stdlib std::sort sort via FFI.
#[cfg(feature = "cpp_std")]
pub mod cpp_std;
