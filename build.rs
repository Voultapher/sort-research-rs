use std::env;
use std::path::PathBuf;

#[cfg(feature = "cpp_pdqsort")]
fn build_and_link_cpp_pdqsort() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cpp_pdqsort_sort_cpp_path = manifest_dir.join("src").join("cpp").join("cpp_pdqsort.cpp");

    // Tell Cargo that if the given file changes, to rerun this build script.
    println!(
        "cargo:rerun-if-changed={}",
        cpp_pdqsort_sort_cpp_path.display()
    );

    println!("{}", cpp_pdqsort_sort_cpp_path.display().to_string());

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    cc::Build::new()
        .file(cpp_pdqsort_sort_cpp_path)
        .cpp(true)
        .warnings(true)
        .flag_if_supported("/EHsc")
        .flag_if_supported("/std:c++20")
        .flag_if_supported("-std=c++20")
        .opt_level(2)
        .compile("cpp_pdqsort");

    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=static={}", "cpp_pdqsort");
}

#[cfg(not(feature = "cpp_pdqsort"))]
fn build_and_link_cpp_pdqsort() {}

#[cfg(feature = "cpp_std")]
fn build_and_link_cpp_std() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cpp_std_src_path = manifest_dir
        .join("src")
        .join("cpp")
        .join("cpp_std_sort.cpp");

    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed={}", cpp_std_src_path.display());

    println!("{}", cpp_std_src_path.display().to_string());

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut build = cc::Build::new();

    build
        .file(cpp_std_src_path)
        .cpp(true)
        .warnings(true)
        .warnings_into_errors(true)
        .flag_if_supported("/EHsc")
        .flag_if_supported("/std:c++20")
        .flag_if_supported("-std=c++20")
        .opt_level(2);

    if cfg!(feature = "libcxx") {
        let libcxx_build_dir = PathBuf::from(
            env::var("LIBCXX_CUSTOM_BUILD_DIR").expect("LIBCXX_CUSTOM_BUILD_DIR env var not set"),
        );

        let libcxx_include_dir = libcxx_build_dir.join("include").join("c++").join("v1");
        let libcxx_lib_path = libcxx_build_dir.join("lib");

        build
            .compiler("clang++")
            .flag("-nostdinc++")
            .flag("-nostdlib++")
            .flag("-isystem")
            .flag(&libcxx_include_dir.display().to_string());

        println!("cargo:rustc-link-search={}", libcxx_lib_path.display());
        println!("cargo:rustc-link-lib=static={}", "c++");
        println!("cargo:rustc-link-lib=static={}", "c++abi");
    }

    build.compile("cpp_std_sort");

    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=static={}", "cpp_std_sort");
}

#[cfg(not(feature = "cpp_std"))]
fn build_and_link_cpp_std() {}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let build_rs_path = manifest_dir.join("build.rs").canonicalize().unwrap();

    // By default without this line, cargo re-runs the build script for all source changes.
    println!("cargo:rerun-if-changed={}", build_rs_path.display());

    build_and_link_cpp_pdqsort();
    build_and_link_cpp_std();
}
