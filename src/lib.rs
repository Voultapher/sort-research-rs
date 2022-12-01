#![allow(incomplete_features)]
#![feature(
    maybe_uninit_uninit_array,
    maybe_uninit_slice,
    core_intrinsics,
    ptr_sub_ptr,
    strict_provenance,
    unchecked_math,
    cell_update,
    specialization,
    sized_type_properties
)]

pub trait Sort {
    fn name() -> String;

    fn sort<T>(arr: &mut [T])
    where
        T: Ord;

    fn sort_by<T, F>(arr: &mut [T], compare: F)
    where
        F: FnMut(&T, &T) -> std::cmp::Ordering;
}

pub mod patterns;

macro_rules! sort_impl {
    ($name:expr) => {
        pub struct SortImpl;

        impl crate::Sort for SortImpl {
            fn name() -> String {
                $name.into()
            }

            #[inline]
            fn sort<T>(arr: &mut [T])
            where
                T: Ord,
            {
                sort(arr);
            }

            #[inline]
            fn sort_by<T, F>(arr: &mut [T], compare: F)
            where
                F: FnMut(&T, &T) -> Ordering,
            {
                sort_by(arr, compare);
            }
        }
    };
}

#[macro_use]
mod ffi_util;

// Copy the stdlib implementations to have comparable builds.
// The stdlib is compiled with unknown optimizations such as PGO.
pub mod other;
pub mod stable;
pub mod unstable;
