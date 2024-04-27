#[cfg(feature = "rust_radsort")]
pub mod rust_radsort;

#[cfg(feature = "rust_afsort")]
pub mod rust_afsort;

// Call simdsort sort via FFI.
#[cfg(feature = "cpp_simdsort")]
pub mod cpp_simdsort;

// Call vqsort sort via FFI.
#[cfg(feature = "cpp_vqsort")]
pub mod cpp_vqsort;

// Call intel_avx512 sort via FFI.
#[cfg(feature = "cpp_intel_avx512")]
pub mod cpp_intel_avx512;

// Call singelisort sort via FFI.
#[cfg(feature = "singeli_singelisort")]
pub mod singeli_singelisort;

#[cfg(feature = "evolution")]
pub mod sort_evolution;

#[cfg(feature = "small_sort")]
pub mod small_sort;

#[cfg(feature = "partition_point")]
pub mod partition_point;

#[cfg(feature = "partition")]
pub mod partition;

#[cfg(feature = "selection")]
pub mod selection;
