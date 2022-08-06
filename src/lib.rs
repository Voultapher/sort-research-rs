#![feature(
    maybe_uninit_uninit_array,
    maybe_uninit_slice,
    core_intrinsics,
    ptr_sub_ptr,
    strict_provenance
)]

pub mod patterns;

pub mod fluxsort;
pub mod new_stable_sort;

// Copy the stdlib implementations to have comparable builds.
// The stdlib is compiled with unknown optimizations such as PGO.
pub mod stdlib_stable;
pub mod stdlib_unstable;
