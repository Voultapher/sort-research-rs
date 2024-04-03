use std::env;

use criterion::{black_box, Criterion};

use sort_test_tools::Sort;

#[allow(unused_imports)]
use sort_research_rs::{other, stable, unstable};

use crate::modules::util;

fn measure_comp_count<S: Sort, T: Ord + std::fmt::Debug>(
    name: &str,
    test_len: usize,
    transform: &fn(Vec<i32>) -> Vec<T>,
    pattern_provider: impl Fn(usize) -> Vec<i32>,
) {
    // Measure how many comparisons are performed by a specific implementation and input
    // combination.
    let run_count: usize = if test_len <= 20 {
        100_000
    } else if test_len < 10_000 {
        3000
    } else if test_len < 100_000 {
        1000
    } else if test_len < 1_000_000 {
        100
    } else {
        10
    };

    let mut comp_count = 0u64;

    // Instrument via sort_by to ensure the type properties such as Copy of the type
    // that is being sorted doesn't change. And we get representative numbers.
    for _ in 0..run_count {
        let mut test_data = transform(pattern_provider(test_len));
        S::sort_by(black_box(test_data.as_mut_slice()), |a, b| {
            comp_count += 1;
            a.cmp(b)
        })
    }

    // If there is on average less than a single comparison this will be wrong.
    // But that's such a corner case I don't care about it.
    let total = comp_count / (run_count as u64);
    println!("{name}: mean comparisons: {total}");
}

pub fn bench_fn<S: Sort, T: Ord + std::fmt::Debug>(
    c: &mut Criterion,
    test_len: usize,
    transform_name: &str,
    transform: &fn(Vec<i32>) -> Vec<T>,
    pattern_name: &str,
    pattern_provider: impl Fn(usize) -> Vec<i32>,
) {
    let bench_name = S::name();

    if env::var("MEASURE_COMP").is_ok() {
        let name = format!(
            "{}-comp-{}-{}-{}",
            bench_name, transform_name, pattern_name, test_len
        );

        if util::should_run_benchmark(&name) {
            measure_comp_count::<S, T>(&name, test_len, transform, pattern_provider);
        }
    } else {
        util::bench_fn(
            c,
            test_len,
            transform_name,
            transform,
            pattern_name,
            pattern_provider,
            &bench_name,
            S::sort,
        );
    }
}

pub fn bench<T: Ord + std::fmt::Debug>(
    c: &mut Criterion,
    test_len: usize,
    transform_name: &str,
    transform: &fn(Vec<i32>) -> Vec<T>,
    pattern_name: &str,
    pattern_provider: &fn(usize) -> Vec<i32>,
) {
    macro_rules! bench_inst {
        ($sort_impl_path:path) => {{
            use $sort_impl_path::*;

            bench_fn::<SortImpl, T>(
                c,
                test_len,
                transform_name,
                transform,
                pattern_name,
                pattern_provider,
            );
        }};
    }

    // --- Stable sorts ---

    bench_inst!(stable::rust_std);

    #[cfg(feature = "cpp_std_sys")]
    bench_inst!(stable::cpp_std_sys);

    #[cfg(feature = "cpp_std_libcxx")]
    bench_inst!(stable::cpp_std_libcxx);

    #[cfg(feature = "cpp_std_gcc4_3")]
    bench_inst!(stable::cpp_std_gcc4_3);

    #[cfg(feature = "cpp_powersort")]
    bench_inst!(stable::cpp_powersort);

    #[cfg(feature = "cpp_powersort")]
    bench_inst!(stable::cpp_powersort_4way);

    #[cfg(feature = "c_fluxsort")]
    bench_inst!(stable::c_fluxsort);

    #[cfg(feature = "golang_std")]
    bench_inst!(stable::golang_std);

    #[cfg(feature = "rust_wpwoodjr")]
    bench_inst!(stable::rust_wpwoodjr);

    #[cfg(feature = "rust_glidesort")]
    bench_inst!(stable::rust_glidesort);

    #[cfg(feature = "rust_driftsort")]
    bench_inst!(stable::rust_driftsort);

    #[cfg(feature = "rust_tinysort")]
    bench_inst!(stable::rust_tinysort);

    // --- Unstable sorts ---

    bench_inst!(unstable::rust_ipnsort);

    bench_inst!(unstable::rust_std);

    #[cfg(feature = "rust_dmsort")]
    bench_inst!(unstable::rust_dmsort);

    #[cfg(feature = "rust_crumsort_rs")]
    bench_inst!(unstable::rust_crumsort_rs);

    #[cfg(feature = "rust_tinysort")]
    bench_inst!(unstable::rust_tinysort);

    #[cfg(feature = "cpp_pdqsort")]
    bench_inst!(unstable::cpp_pdqsort);

    #[cfg(feature = "cpp_ips4o")]
    bench_inst!(unstable::cpp_ips4o);

    #[cfg(feature = "cpp_blockquicksort")]
    bench_inst!(unstable::cpp_blockquicksort);

    #[cfg(feature = "cpp_gerbens_qsort")]
    bench_inst!(unstable::cpp_gerbens_qsort);

    #[cfg(feature = "cpp_nanosort")]
    bench_inst!(unstable::cpp_nanosort);

    #[cfg(feature = "c_std_sys")]
    bench_inst!(unstable::c_std_sys);

    #[cfg(feature = "c_crumsort")]
    bench_inst!(unstable::c_crumsort);

    #[cfg(feature = "cpp_std_sys")]
    bench_inst!(unstable::cpp_std_sys);

    #[cfg(feature = "cpp_std_libcxx")]
    bench_inst!(unstable::cpp_std_libcxx);

    #[cfg(feature = "cpp_std_gcc4_3")]
    bench_inst!(unstable::cpp_std_gcc4_3);

    #[cfg(feature = "golang_std")]
    bench_inst!(unstable::golang_std);

    // --- Other sorts ---

    #[cfg(feature = "rust_radsort")]
    bench_inst!(other::rust_radsort);

    #[cfg(feature = "cpp_simdsort")]
    bench_inst!(other::cpp_simdsort);

    #[cfg(feature = "cpp_vqsort")]
    bench_inst!(other::cpp_vqsort);

    #[cfg(feature = "cpp_intel_avx512")]
    bench_inst!(other::cpp_intel_avx512);

    #[cfg(feature = "singeli_singelisort")]
    bench_inst!(other::singeli_singelisort);

    #[cfg(feature = "evolution")]
    {
        bench_inst!(other::sort_evolution::stable::timsort_evo0);
        bench_inst!(other::sort_evolution::stable::timsort_evo1);
        bench_inst!(other::sort_evolution::stable::timsort_evo2);
        bench_inst!(other::sort_evolution::stable::timsort_evo3);
        bench_inst!(other::sort_evolution::stable::timsort_evo4);

        bench_inst!(other::sort_evolution::unstable::quicksort_evo0);
        bench_inst!(other::sort_evolution::unstable::quicksort_stack_evo0);

        bench_inst!(other::sort_evolution::other::bucket_btree);
        bench_inst!(other::sort_evolution::other::bucket_hash);
        bench_inst!(other::sort_evolution::other::bucket_match);
        bench_inst!(other::sort_evolution::other::bucket_branchless);
        bench_inst!(other::sort_evolution::other::bucket_phf);
    }

    #[cfg(feature = "small_sort")]
    {
        bench_inst!(other::small_sort::sort4_unstable_cmp_swap);
        bench_inst!(other::small_sort::sort4_unstable_ptr_select);
        bench_inst!(other::small_sort::sort4_unstable_branchy);
        bench_inst!(other::small_sort::sort4_stable_orson);
        bench_inst!(other::small_sort::sort10_unstable_cmp_swaps);
        bench_inst!(other::small_sort::sort10_unstable_experimental);
        bench_inst!(other::small_sort::sort10_unstable_ptr_select);
    }

    #[cfg(feature = "selection")]
    {
        bench_inst!(other::selection::rust_ipnsort);
        bench_inst!(other::selection::rust_std);
    }
}
