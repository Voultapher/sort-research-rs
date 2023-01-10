use std::env;
use std::path::PathBuf;

#[allow(dead_code)]
fn link_simple_cpp_sort(file_name: &str, specialize_fn: Option<fn(&mut cc::Build)>) {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let file_path = manifest_dir
        .join("src")
        .join("cpp")
        .join(format!("{file_name}.cpp"));

    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed={}", file_path.display());

    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir
            .join("src")
            .join("cpp")
            .join("shared.h")
            .display()
    );

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut builder = cc::Build::new();

    builder
        .file(file_path)
        .cpp(true)
        .warnings(false) // The thirdparties just have too many.
        .flag_if_supported("/EHsc")
        .flag_if_supported("/std:c++20")
        .flag_if_supported("-std=c++20")
        .flag_if_supported("-fdiagnostics-color=always")
        .force_frame_pointer(false)
        .define("NDEBUG", None)
        .opt_level(3);

    if let Some(spec_fn) = specialize_fn {
        spec_fn(&mut builder);
    }

    builder.compile(file_name);

    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=static={}", file_name);
}

#[cfg(feature = "cpp_pdqsort")]
fn build_and_link_cpp_pdqsort() {
    link_simple_cpp_sort("cpp_pdqsort", None);
}

#[cfg(not(feature = "cpp_pdqsort"))]
fn build_and_link_cpp_pdqsort() {}

#[cfg(feature = "cpp_powersort")]
fn build_and_link_cpp_powersort() {
    link_simple_cpp_sort("cpp_powersort", None);
}

#[cfg(not(feature = "cpp_powersort"))]
fn build_and_link_cpp_powersort() {}

#[cfg(feature = "cpp_simdsort")]
fn build_and_link_cpp_simdsort() {
    link_simple_cpp_sort(
        "cpp_simdsort",
        Some(|builder: &mut cc::Build| {
            // Make an exception for march=native here because AVX2 will not work without it.
            builder.flag_if_supported("-march=native");
        }),
    );
}

#[cfg(not(feature = "cpp_simdsort"))]
fn build_and_link_cpp_simdsort() {}

#[cfg(feature = "cpp_ips4o")]
fn build_and_link_cpp_ips4o() {
    link_simple_cpp_sort("cpp_ips4o", None);
}

#[cfg(not(feature = "cpp_ips4o"))]
fn build_and_link_cpp_ips4o() {}

#[cfg(feature = "cpp_blockquicksort")]
fn build_and_link_cpp_blockquicksort() {
    link_simple_cpp_sort("cpp_blockquicksort", None);
}

#[cfg(not(feature = "cpp_blockquicksort"))]
fn build_and_link_cpp_blockquicksort() {}

#[cfg(feature = "c_crumsort")]
fn build_and_link_c_crumsort() {
    link_simple_cpp_sort("c_crumsort", None);
}

#[cfg(not(feature = "c_crumsort"))]
fn build_and_link_c_crumsort() {}

#[cfg(feature = "c_fluxsort")]
fn build_and_link_c_fluxsort() {
    link_simple_cpp_sort("c_fluxsort", None);
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
        .define("NDEBUG", None)
        .opt_level(3)
        .force_frame_pointer(false)
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
        .define("NDEBUG", None)
        .opt_level(3)
        .force_frame_pointer(false)
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
    build_and_link_cpp_powersort();
    build_and_link_cpp_simdsort();
    build_and_link_cpp_ips4o();
    build_and_link_cpp_blockquicksort();
    build_and_link_c_crumsort();
    build_and_link_c_fluxsort();
    build_and_link_cpp_std_sys();
    build_and_link_cpp_std_libcxx();
}
