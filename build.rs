use std::env;
use std::path::PathBuf;

// Adjust this if you have a custom clang build, or path.
#[allow(unused)]
const CLANG_PATH: &str = "clang++";

#[allow(dead_code)]
fn build_and_link_cpp_sort(
    file_name: &str,
    specialize_fn: Option<fn(&mut cc::Build) -> Option<String>>,
) {
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
        .flag_if_supported("/Zc:__cplusplus")
        .flag_if_supported("/std:c++20")
        .flag_if_supported("-std=c++20")
        .flag_if_supported("-fdiagnostics-color=always")
        .force_frame_pointer(false)
        .define("NDEBUG", None)
        .debug(false)
        .opt_level(3);

    let mut artifact_name = file_name.to_string();
    if let Some(spec_fn) = specialize_fn {
        if let Some(artifact_name_override) = spec_fn(&mut builder) {
            artifact_name = artifact_name_override;
        }
    }

    builder.compile(&artifact_name);

    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=static={}", artifact_name);
}

#[cfg(feature = "cpp_pdqsort")]
fn build_and_link_cpp_pdqsort() {
    build_and_link_cpp_sort("cpp_pdqsort", None);
}

#[cfg(not(feature = "cpp_pdqsort"))]
fn build_and_link_cpp_pdqsort() {}

#[cfg(feature = "cpp_powersort")]
fn build_and_link_cpp_powersort() {
    build_and_link_cpp_sort("cpp_powersort", None);
}

#[cfg(not(feature = "cpp_powersort"))]
fn build_and_link_cpp_powersort() {}

#[cfg(feature = "cpp_simdsort")]
fn build_and_link_cpp_simdsort() {
    build_and_link_cpp_sort(
        "cpp_simdsort",
        Some(|builder: &mut cc::Build| {
            // Make an exception for march=native here because AVX2 will not work without it.
            builder.flag_if_supported("-march=native");

            None
        }),
    );
}

#[cfg(not(feature = "cpp_simdsort"))]
fn build_and_link_cpp_simdsort() {}

#[cfg(feature = "cpp_ips4o")]
fn build_and_link_cpp_ips4o() {
    build_and_link_cpp_sort("cpp_ips4o", None);
}

#[cfg(feature = "cpp_vqsort")]
fn build_and_link_cpp_vqsort() {
    build_and_link_cpp_sort(
        "cpp_vqsort",
        Some(|builder: &mut cc::Build| {
            // Make an exception for march=native here because AVX2 will not work without it.
            builder.flag("-march=native");
            builder.compiler(CLANG_PATH); // gcc yields significantly worse code-gen here.

            None
        }),
    );
}

#[cfg(not(feature = "cpp_vqsort"))]
fn build_and_link_cpp_vqsort() {}

#[cfg(feature = "cpp_intel_avx512")]
fn build_and_link_cpp_intel_avx512() {
    build_and_link_cpp_sort(
        "cpp_intel_avx512",
        Some(|builder: &mut cc::Build| {
            // Make an exception for march=native here because AVX512 will not work without it.
            builder.flag("-march=native");
            builder.compiler(CLANG_PATH); // gcc yields significantly worse code-gen here.

            None
        }),
    );
}

#[cfg(not(feature = "cpp_intel_avx512"))]
fn build_and_link_cpp_intel_avx512() {}

#[cfg(feature = "singeli_singelisort")]
fn build_and_link_singelisort() {
    build_and_link_cpp_sort(
        "singeli_singelisort",
        Some(|builder: &mut cc::Build| {
            // Clang seems to produce slightly better perf.
            builder.compiler(CLANG_PATH);

            None
        }),
    );
}

#[cfg(not(feature = "singeli_singelisort"))]
fn build_and_link_singelisort() {}

#[cfg(feature = "golang_std")]
fn build_and_link_golang_std() {
    use std::fs;
    use std::process::Command;

    build_and_link_cpp_sort(
        "golang_std",
        Some(|builder: &mut cc::Build| {
            let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
            let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

            let go_ffi_lib_file_path = manifest_dir
                .join("src")
                .join("cpp")
                .join(format!("golang_std_ffi_lib.go"));

            println!("cargo:rerun-if-changed={}", go_ffi_lib_file_path.display());

            let cmd_output = Command::new("go")
                .args([
                    "build",
                    "-buildmode=c-archive",
                    &go_ffi_lib_file_path.display().to_string(),
                ])
                .current_dir(&out_dir)
                .output()
                .expect("failed to execute process");

            if !cmd_output.status.success() {
                eprintln!("{}", String::from_utf8(cmd_output.stderr).unwrap());
                panic!();
            }

            let golang_std_ffi_lib_path = out_dir.join("golang_std_ffi_lib.a");
            if !golang_std_ffi_lib_path.exists() {
                panic!("go build did not produce static library as expected");
            }

            fs::rename(
                &golang_std_ffi_lib_path,
                out_dir.join("libgolang_std_ffi.a"),
            )
            .unwrap();

            println!("cargo:rustc-link-search={}", out_dir.display());
            println!("cargo:rustc-link-lib=static=golang_std_ffi");

            builder.include(&out_dir);

            None
        }),
    );
}

#[cfg(not(feature = "golang_std"))]
fn build_and_link_golang_std() {}

#[cfg(not(feature = "cpp_ips4o"))]
fn build_and_link_cpp_ips4o() {}

#[cfg(feature = "cpp_blockquicksort")]
fn build_and_link_cpp_blockquicksort() {
    build_and_link_cpp_sort("cpp_blockquicksort", None);
}

#[cfg(not(feature = "cpp_blockquicksort"))]
fn build_and_link_cpp_blockquicksort() {}

#[cfg(feature = "cpp_gerbens_qsort")]
fn build_and_link_cpp_gerbens_qsort() {
    build_and_link_cpp_sort(
        "cpp_gerbens_qsort",
        Some(|builder: &mut cc::Build| {
            builder.compiler(CLANG_PATH); // gcc yields significantly worse code-gen here.

            None
        }),
    );
}

#[cfg(not(feature = "cpp_gerbens_qsort"))]
fn build_and_link_cpp_gerbens_qsort() {}

#[cfg(feature = "cpp_nanosort")]
fn build_and_link_cpp_nanosort() {
    build_and_link_cpp_sort(
        "cpp_nanosort",
        Some(|builder: &mut cc::Build| {
            builder.compiler(CLANG_PATH); // gcc yields significantly worse code-gen here.

            None
        }),
    );
}

#[cfg(not(feature = "cpp_nanosort"))]
fn build_and_link_cpp_nanosort() {}

#[cfg(feature = "cpp_wikisort")]
fn build_and_link_cpp_wikisort() {
    build_and_link_cpp_sort(
        "cpp_wikisort",
        Some(|builder: &mut cc::Build| {
            // clang yields better code-gen for random patterns, gcc for partially sorted ones.
            builder.compiler(CLANG_PATH);

            None
        }),
    );
}

#[cfg(not(feature = "cpp_wikisort"))]
fn build_and_link_cpp_wikisort() {}

#[cfg(feature = "c_std_sys")]
fn build_and_link_c_std_sys() {
    build_and_link_cpp_sort("c_std_sys", None);
}

#[cfg(not(feature = "c_std_sys"))]
fn build_and_link_c_std_sys() {}

#[cfg(feature = "c_llvm_libc")]
fn build_and_link_c_llvm_libc() {
    build_and_link_cpp_sort(
        "c_llvm_libc",
        Some(|builder: &mut cc::Build| {
            // It's a clang associated lib, so clang is expected to generate better code.
            builder.compiler(CLANG_PATH);

            None
        }),
    );
}

#[cfg(not(feature = "c_llvm_libc"))]
fn build_and_link_c_llvm_libc() {}

#[cfg(feature = "c_idisort")]
fn build_and_link_c_idisort() {
    build_and_link_cpp_sort(
        "c_idisort",
        Some(|builder: &mut cc::Build| {
            // Designed and tested with clang.
            builder.compiler(CLANG_PATH);

            None
        }),
    );
}

#[cfg(not(feature = "c_idisort"))]
fn build_and_link_c_idisort() {}

#[cfg(feature = "c_crumsort")]
fn build_and_link_c_crumsort() {
    build_and_link_cpp_sort(
        "c_crumsort",
        Some(|builder: &mut cc::Build| {
            builder.compiler(CLANG_PATH); // clang can generate cmov which yields better perf.

            None
        }),
    );
}

#[cfg(not(feature = "c_crumsort"))]
fn build_and_link_c_crumsort() {}

#[cfg(feature = "c_fluxsort")]
fn build_and_link_c_fluxsort() {
    build_and_link_cpp_sort(
        "c_fluxsort",
        Some(|builder: &mut cc::Build| {
            builder.compiler(CLANG_PATH); // clang can generate cmov which yields better perf.

            None
        }),
    );
}

#[cfg(not(feature = "c_fluxsort"))]
fn build_and_link_c_fluxsort() {}

#[cfg(feature = "cpp_std_sys")]
fn build_and_link_cpp_std_sys() {
    build_and_link_cpp_sort(
        "cpp_std_sort",
        Some(|builder| {
            builder.define("STD_LIB_SYS", None);

            Some("cpp_std_sort_sys".into())
        }),
    );
}

#[cfg(not(feature = "cpp_std_sys"))]
fn build_and_link_cpp_std_sys() {}

#[cfg(feature = "cpp_std_libcxx")]
fn build_and_link_cpp_std_libcxx() {
    build_and_link_cpp_sort(
        "cpp_std_sort",
        Some(|builder| {
            builder
                .define("STD_LIB_LIBCXX", None)
                .compiler(CLANG_PATH)
                .cpp_set_stdlib("c++"); // Use libcxx

            Some("cpp_std_sort_libcxx".into())
        }),
    );
}

#[cfg(not(feature = "cpp_std_libcxx"))]
fn build_and_link_cpp_std_libcxx() {}

#[cfg(feature = "cpp_std_gcc4_3")]
fn build_and_link_cpp_std_gcc4_3() {
    build_and_link_cpp_sort(
        "cpp_std_gcc4_3_sort",
        Some(|builder| {
            let gcc4_3_build_dir =
                env::var("GCC4_3_BUILD_DIR").expect("GCC4_3_BUILD_DIR env var not set");

            let compiler_path = PathBuf::from(gcc4_3_build_dir)
                .join("usr")
                .join("bin")
                .join("g++-4.3");

            builder.compiler(compiler_path).flag("-std=gnu++0x");

            None
        }),
    );
}

#[cfg(not(feature = "cpp_std_gcc4_3"))]
fn build_and_link_cpp_std_gcc4_3() {}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let build_rs_path = manifest_dir.join("build.rs").canonicalize().unwrap();

    // By default without this line, cargo re-runs the build script for all source changes.
    println!("cargo:rerun-if-changed={}", build_rs_path.display());

    build_and_link_cpp_pdqsort();
    build_and_link_cpp_powersort();
    build_and_link_cpp_simdsort();
    build_and_link_cpp_vqsort();
    build_and_link_cpp_intel_avx512();
    build_and_link_singelisort();
    build_and_link_golang_std();
    build_and_link_cpp_ips4o();
    build_and_link_cpp_blockquicksort();
    build_and_link_cpp_gerbens_qsort();
    build_and_link_cpp_nanosort();
    build_and_link_cpp_wikisort();
    build_and_link_c_std_sys();
    build_and_link_c_llvm_libc();
    build_and_link_c_idisort();
    build_and_link_c_crumsort();
    build_and_link_c_fluxsort();
    build_and_link_cpp_std_sys();
    build_and_link_cpp_std_libcxx();
    build_and_link_cpp_std_gcc4_3();
}
