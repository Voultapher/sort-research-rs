#include "thirdparty/llvm_libc/qsort.h"

#include <stdint.h>
#include <stdexcept>

#include "shared.h"

template <typename T>
uint32_t sort_by_impl(T* data,
                      size_t len,
                      CompResult (*cmp_fn)(const T&, const T&, uint8_t*),
                      uint8_t* ctx) noexcept {
    try {
        LIBC_NAMESPACE_DECL::qsort(static_cast<void*>(data), len, sizeof(T),
                                   make_compare_fn_c(cmp_fn, ctx));
    } catch (...) {
        return 1;
    }

    return 0;
}

extern "C" {
// --- i32 ---

void qsort_llvm_libc_unstable_i32(int32_t* data, size_t len) {
    LIBC_NAMESPACE_DECL::qsort(static_cast<void*>(data), len, sizeof(int32_t),
                               int_cmp_func<int32_t>);
}

uint32_t qsort_llvm_libc_unstable_i32_by(int32_t* data,
                                         size_t len,
                                         CompResult (*cmp_fn)(const int32_t&,
                                                              const int32_t&,
                                                              uint8_t*),
                                         uint8_t* ctx) {
    return sort_by_impl(data, len, cmp_fn, ctx);
}

// --- u64 ---

void qsort_llvm_libc_unstable_u64(uint64_t* data, size_t len) {
    LIBC_NAMESPACE_DECL::qsort(static_cast<void*>(data), len, sizeof(uint64_t),
                               int_cmp_func<uint64_t>);
}

uint32_t qsort_llvm_libc_unstable_u64_by(uint64_t* data,
                                         size_t len,
                                         CompResult (*cmp_fn)(const uint64_t&,
                                                              const uint64_t&,
                                                              uint8_t*),
                                         uint8_t* ctx) {
    return sort_by_impl(data, len, cmp_fn, ctx);
}

// --- ffi_string ---

void qsort_llvm_libc_unstable_ffi_string(FFIString* data, size_t len) {
    static_assert(sizeof(FFIString) == sizeof(FFIStringCpp));
    LIBC_NAMESPACE_DECL::qsort(static_cast<void*>(data), len, sizeof(FFIString),
                               int_cmp_func<FFIStringCpp>);
}

uint32_t qsort_llvm_libc_unstable_ffi_string_by(FFIString* data,
                                                size_t len,
                                                CompResult (*cmp_fn)(const FFIString&,
                                                                     const FFIString&,
                                                                     uint8_t*),
                                                uint8_t* ctx) {
    return sort_by_impl(data, len, cmp_fn, ctx);
}

// --- f128 ---

void qsort_llvm_libc_unstable_f128(F128* data, size_t len) {
    static_assert(sizeof(F128) == sizeof(F128Cpp));
    LIBC_NAMESPACE_DECL::qsort(static_cast<void*>(data), len, sizeof(F128), int_cmp_func<F128Cpp>);
}

uint32_t qsort_llvm_libc_unstable_f128_by(F128* data,
                                          size_t len,
                                          CompResult (*cmp_fn)(const F128&, const F128&, uint8_t*),
                                          uint8_t* ctx) {
    return sort_by_impl(data, len, cmp_fn, ctx);
}

// --- 1k ---

void qsort_llvm_libc_unstable_1k(FFIOneKibiByte* data, size_t len) {
    static_assert(sizeof(FFIOneKibiByte) == sizeof(FFIOneKibiByteCpp));
    LIBC_NAMESPACE_DECL::qsort(static_cast<void*>(data), len, sizeof(FFIOneKibiByte),
                               int_cmp_func<FFIOneKibiByteCpp>);
}

uint32_t qsort_llvm_libc_unstable_1k_by(FFIOneKibiByte* data,
                                        size_t len,
                                        CompResult (*cmp_fn)(const FFIOneKibiByte&,
                                                             const FFIOneKibiByte&,
                                                             uint8_t*),
                                        uint8_t* ctx) {
    return sort_by_impl(data, len, cmp_fn, ctx);
}
}  // extern "C"
