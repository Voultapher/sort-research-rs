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
    sized_type_properties,
    portable_simd,
    const_mut_refs,
    vec_into_raw_parts,
    inline_const,
    const_trait_impl,
    negative_impls,
    auto_traits,
    generic_const_exprs
)]

macro_rules! sort_impl {
    ($name:expr) => {
        pub struct SortImpl;

        impl sort_test_tools::Sort for SortImpl {
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
pub mod ffi_util;

// Copy the stdlib implementations to have comparable builds.
// The stdlib is compiled with unknown optimizations such as PGO.
pub mod other;
pub mod stable;
pub mod unstable;
