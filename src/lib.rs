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

#[macro_use]
mod ffi_util;

// Copy the stdlib implementations to have comparable builds.
// The stdlib is compiled with unknown optimizations such as PGO.
pub mod other;
pub mod stable;
pub mod unstable;
