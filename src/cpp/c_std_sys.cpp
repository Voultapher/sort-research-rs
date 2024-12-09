#include <stdint.h>
#include <stdlib.h>
#include <stdexcept>

#include "shared.h"

template <typename T>
uint32_t sort_by_impl(T* data,
                      size_t len,
                      CompResult (*cmp_fn)(const T&, const T&, uint8_t*),
                      uint8_t* ctx) noexcept {
    try {
        qsort(static_cast<void*>(data), len, sizeof(T), make_compare_fn_c(cmp_fn, ctx));
    } catch (...) {
        return 1;
    }

    return 0;
}

extern "C" {
// --- i32 ---

void qsort_unstable_i32(int32_t* data, size_t len) {
    qsort(static_cast<void*>(data), len, sizeof(int32_t), int_cmp_func<int32_t>);
}

uint32_t qsort_unstable_i32_by(int32_t* data,
                               size_t len,
                               CompResult (*cmp_fn)(const int32_t&, const int32_t&, uint8_t*),
                               uint8_t* ctx) {
    return sort_by_impl(data, len, cmp_fn, ctx);
}

// --- u64 ---

void qsort_unstable_u64(uint64_t* data, size_t len) {
    qsort(static_cast<void*>(data), len, sizeof(uint64_t), int_cmp_func<uint64_t>);
}

uint32_t qsort_unstable_u64_by(uint64_t* data,
                               size_t len,
                               CompResult (*cmp_fn)(const uint64_t&, const uint64_t&, uint8_t*),
                               uint8_t* ctx) {
    return sort_by_impl(data, len, cmp_fn, ctx);
}

// --- ffi_string ---

void qsort_unstable_ffi_string(FFIString* data, size_t len) {
    // Value would have to be sorted by indirection.
    printf("Not supported\n");
}

uint32_t qsort_unstable_ffi_string_by(FFIString* data,
                                      size_t len,
                                      CompResult (*cmp_fn)(const FFIString&,
                                                           const FFIString&,
                                                           uint8_t*),
                                      uint8_t* ctx) {
    printf("Not supported\n");
    return 1;
}

// --- f128 ---

void qsort_unstable_f128(F128* data, size_t len) {
    // Swaps values incorrectly, or my implementation is wrong.
    printf("Not supported\n");
}

uint32_t qsort_unstable_f128_by(F128* data,
                                size_t len,
                                CompResult (*cmp_fn)(const F128&, const F128&, uint8_t*),
                                uint8_t* ctx) {
    printf("Not supported\n");
    return 1;
}

// --- 1k ---

void qsort_unstable_1k(FFIOneKibiByte* data, size_t len) {
    // Value would have to be sorted by indirection.
    printf("Not supported\n");
}

uint32_t qsort_unstable_1k_by(FFIOneKibiByte* data,
                              size_t len,
                              CompResult (*cmp_fn)(const FFIOneKibiByte&,
                                                   const FFIOneKibiByte&,
                                                   uint8_t*),
                              uint8_t* ctx) {
    printf("Not supported\n");
    return 1;
}
}  // extern "C"
