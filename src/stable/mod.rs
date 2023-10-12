pub mod rust_std;

#[cfg(feature = "rust_wpwoodjr")]
pub mod rust_wpwoodjr;

#[cfg(feature = "rust_glidesort")]
pub mod rust_glidesort;

#[cfg(feature = "rust_tinysort")]
pub mod rust_tinysort;

// Call stdlib std::sort_stable sort via FFI.
#[cfg(feature = "cpp_std_sys")]
pub mod cpp_std_sys;

// Call stdlib std::sort_stable sort via FFI.
#[cfg(feature = "cpp_std_libcxx")]
pub mod cpp_std_libcxx;

// Call stdlib std::sort_stable sort via FFI.
#[cfg(feature = "cpp_std_gcc4_3")]
pub mod cpp_std_gcc4_3;

// Call powersort sort via FFI.
#[cfg(feature = "cpp_powersort")]
pub mod cpp_powersort;

// Call powersort_4way sort via FFI.
#[cfg(feature = "cpp_powersort")]
pub mod cpp_powersort_4way;

// Call fluxsort sort via FFI.
// Note, this sort is only stable if the the supplied comparison returns less, equal and more.
#[cfg(feature = "c_fluxsort")]
pub mod c_fluxsort;

#[cfg(feature = "rust_rayon_parallel")]
pub mod rust_rayon_parallel;
