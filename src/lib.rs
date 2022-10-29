// TODO figure out Copy specialization with min_specialization.
#![allow(incomplete_features)]
#![feature(
    maybe_uninit_uninit_array,
    maybe_uninit_slice,
    core_intrinsics,
    ptr_sub_ptr,
    strict_provenance,
    unchecked_math,
    cell_update,
    specialization
)]

pub mod patterns;

pub mod fluxsort;
pub mod new_stable_sort;
pub mod new_unstable_sort;

#[cfg(feature = "wpwoodjr")]
pub mod wpwoodjr_stable_sort;

#[cfg(feature = "bevy_radsort")]
pub mod radsort;

#[cfg(feature = "emilk_dmsort")]
pub mod dmsort;

// Copy the stdlib implementations to have comparable builds.
// The stdlib is compiled with unknown optimizations such as PGO.
pub mod stdlib_stable;
pub mod stdlib_unstable;

#[cfg(any(feature = "libcxx", feature = "cpp_pdqsort"))]
#[macro_use]
mod ffi_util;

// Call libcxx sort via FFI.
#[cfg(feature = "libcxx")]
pub mod libcxx_stable;
#[cfg(feature = "libcxx")]
pub mod libcxx_unstable;

// Call pdqsort sort via FFI.
#[cfg(feature = "cpp_pdqsort")]
pub mod cpp_pdqsort;
