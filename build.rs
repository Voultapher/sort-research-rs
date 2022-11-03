use std::env;
use std::path::PathBuf;

#[allow(dead_code)]
fn link_simple_cpp_sort(file_name: &str) {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let file_path = manifest_dir
        .join("src")
        .join("cpp")
        .join(format!("{file_name}.cpp"));

    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed={}", file_path.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    cc::Build::new()
        .file(file_path)
        .cpp(true)
        .warnings(false) // The thirdparties just have too many.
        .flag_if_supported("/EHsc")
        .flag_if_supported("/std:c++20")
        .flag_if_supported("-std=c++20")
        .flag_if_supported("-fdiagnostics-color=always")
        .opt_level(2)
        .compile(file_name);

    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=static={}", file_name);
}

#[cfg(feature = "cpp_pdqsort")]
fn build_and_link_cpp_pdqsort() {
    link_simple_cpp_sort("cpp_pdqsort");
}

#[cfg(not(feature = "cpp_pdqsort"))]
fn build_and_link_cpp_pdqsort() {}

#[cfg(feature = "c_crumsort")]
fn build_and_link_c_crumsort() {
    link_simple_cpp_sort("c_crumsort");
}

#[cfg(not(feature = "c_crumsort"))]
fn build_and_link_c_crumsort() {}

#[cfg(feature = "c_fluxsort")]
fn build_and_link_c_fluxsort() {
    link_simple_cpp_sort("c_fluxsort");
}

#[cfg(not(feature = "c_fluxsort"))]
fn build_and_link_c_fluxsort() {}

#[cfg(feature = "cpp_std_sys")]
fn build_and_link_cpp_std_sys() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cpp_std_src_path = manifest_dir
        .join("src")
        .join("cpp")
        .join("cpp_std_sort.cpp");

    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed={}", cpp_std_src_path.display());

    println!("{}", cpp_std_src_path.display().to_string());

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    cc::Build::new()
        .file(cpp_std_src_path)
        .cpp(true)
        .warnings(true)
        .warnings_into_errors(true)
        .flag_if_supported("/EHsc")
        .flag_if_supported("/std:c++20")
        .flag_if_supported("-std=c++20")
        .define("STD_LIB_SYS", None)
        .opt_level(2)
        .compile("cpp_std_sort_sys");

    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=static={}", "cpp_std_sort_sys");
}

#[cfg(not(feature = "cpp_std_sys"))]
fn build_and_link_cpp_std_sys() {}

#[cfg(feature = "cpp_std_libcxx")]
fn build_and_link_cpp_std_libcxx() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cpp_std_src_path = manifest_dir
        .join("src")
        .join("cpp")
        .join("cpp_std_sort.cpp");

    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed={}", cpp_std_src_path.display());

    println!("{}", cpp_std_src_path.display().to_string());

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let libcxx_build_dir = PathBuf::from(
        env::var("LIBCXX_CUSTOM_BUILD_DIR").expect("LIBCXX_CUSTOM_BUILD_DIR env var not set"),
    );

    let libcxx_include_dir = libcxx_build_dir.join("include").join("c++").join("v1");
    let libcxx_lib_path = libcxx_build_dir.join("lib");

    cc::Build::new()
        .file(cpp_std_src_path)
        .cpp(true)
        .warnings(true)
        .warnings_into_errors(true)
        .flag_if_supported("/EHsc")
        .flag_if_supported("/std:c++20")
        .flag_if_supported("-std=c++20")
        .define("STD_LIB_LIBCXX", None)
        .opt_level(2)
        .compiler("clang++")
        .flag("-nostdinc++")
        .flag("-nostdlib++")
        .flag("-isystem")
        .flag(&libcxx_include_dir.display().to_string())
        .compile("cpp_std_sort");

    println!("cargo:rustc-link-search={}", libcxx_lib_path.display());
    println!("cargo:rustc-link-lib=static={}", "c++");
    println!("cargo:rustc-link-lib=static={}", "c++abi");

    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=static={}", "cpp_std_sort");
}

#[cfg(not(feature = "cpp_std_libcxx"))]
fn build_and_link_cpp_std_libcxx() {}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let build_rs_path = manifest_dir.join("build.rs").canonicalize().unwrap();

    // By default without this line, cargo re-runs the build script for all source changes.
    println!("cargo:rerun-if-changed={}", build_rs_path.display());

    build_and_link_cpp_pdqsort();
    build_and_link_c_crumsort();
    build_and_link_c_fluxsort();
    build_and_link_cpp_std_sys();
    build_and_link_cpp_std_libcxx();
}
