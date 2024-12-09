#include "thirdparty/scandum/fluxsort.h"

#include <stdexcept>

#include <stdint.h>

#include "shared.h"

template <typename T>
uint32_t sort_by_impl(T* data,
                      size_t len,
                      CompResult (*cmp_fn)(const T&, const T&, uint8_t*),
                      uint8_t* ctx) noexcept {
    try {
        fluxsort(static_cast<void*>(data), len, sizeof(T), make_compare_fn_c(cmp_fn, ctx));
    } catch (...) {
        return 1;
    }

    return 0;
}

extern "C" {
// --- i32 ---

void fluxsort_stable_i32(int32_t* data, size_t len) {
    fluxsort_prim(static_cast<void*>(data), len, /*signed int*/ 4);
}

uint32_t fluxsort_stable_i32_by(int32_t* data,
                                size_t len,
                                CompResult (*cmp_fn)(const int32_t&, const int32_t&, uint8_t*),
                                uint8_t* ctx) {
    return sort_by_impl(data, len, cmp_fn, ctx);
}

// --- u64 ---

void fluxsort_stable_u64(uint64_t* data, size_t len) {
    fluxsort_prim(static_cast<void*>(data), len, /*unsigned long long*/ 9);
}

uint32_t fluxsort_stable_u64_by(uint64_t* data,
                                size_t len,
                                CompResult (*cmp_fn)(const uint64_t&, const uint64_t&, uint8_t*),
                                uint8_t* ctx) {
    return sort_by_impl(data, len, cmp_fn, ctx);
}

// --- ffi_string ---

void fluxsort_stable_ffi_string(FFIString* data, size_t len) {
    // Value would have to be sorted by indirection.
    printf("Not supported\n");
}

uint32_t fluxsort_stable_ffi_string_by(FFIString* data,
                                       size_t len,
                                       CompResult (*cmp_fn)(const FFIString&,
                                                            const FFIString&,
                                                            uint8_t*),
                                       uint8_t* ctx) {
    printf("Not supported\n");
    return 1;
}

// --- f128 ---

void fluxsort_stable_f128(F128* data, size_t len) {
    // Swaps values incorrectly, or my implementation is wrong.
    printf("Not supported\n");
}

uint32_t fluxsort_stable_f128_by(F128* data,
                                 size_t len,
                                 CompResult (*cmp_fn)(const F128&, const F128&, uint8_t*),
                                 uint8_t* ctx) {
    printf("Not supported\n");
    return 1;
}

// --- 1k ---

void fluxsort_stable_1k(FFIOneKibiByte* data, size_t len) {
    // Value would have to be sorted by indirection.
    printf("Not supported\n");
}

uint32_t fluxsort_stable_1k_by(FFIOneKibiByte* data,
                               size_t len,
                               CompResult (*cmp_fn)(const FFIOneKibiByte&,
                                                    const FFIOneKibiByte&,
                                                    uint8_t*),
                               uint8_t* ctx) {
    printf("Not supported\n");
    return 1;
}
}  // extern "C"
