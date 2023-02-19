#[cfg(feature = "rust_radsort")]
pub mod rust_radsort;

// Call simdsort sort via FFI.
#[cfg(feature = "cpp_simdsort")]
pub mod cpp_simdsort;

// Call vqsort sort via FFI.
#[cfg(feature = "cpp_vqsort")]
pub mod cpp_vqsort;

// Call intel_avx512 sort via FFI.
#[cfg(feature = "cpp_intel_avx512")]
pub mod cpp_intel_avx512;

#[cfg(feature = "evolution")]
pub mod sort_evolution;

pub mod partition;
