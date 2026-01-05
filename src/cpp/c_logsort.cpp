#include <stdint.h>
#include <stdlib.h>

#include <stdexcept>

#include "shared.h"

thread_local static CMPFUNC* thread_local_cmp_fn = nullptr;

#define VAR int32_t

#define CMP(a, b) int_cmp_func<VAR>(a, b)
#define FUNC(NAME) NAME##i32_prim
#include "thirdparty/logsort/logsort.c"
#undef FUNC
#undef CMP

#define CMP(a, b) thread_local_cmp_fn(a, b)
#define FUNC(NAME) NAME##i32
#include "thirdparty/logsort/logsort.c"
#undef FUNC
#undef CMP

#undef VAR

#define VAR uint64_t

#define CMP(a, b) int_cmp_func<VAR>(a, b)
#define FUNC(NAME) NAME##u64_prim
#include "thirdparty/logsort/logsort.c"
#undef FUNC
#undef CMP

#define CMP(a, b) thread_local_cmp_fn(a, b)
#define FUNC(NAME) NAME##u64
#include "thirdparty/logsort/logsort.c"
#undef FUNC
#undef CMP

#undef VAR

constexpr size_t BUF_LEN = 64;

template <typename T>
uint32_t sort_by_impl(T* data,
                      size_t len,
                      CompResult (*cmp_fn)(const T&, const T&, uint8_t*),
                      uint8_t* ctx) noexcept {
    try {
        thread_local_cmp_fn = make_compare_fn_c(cmp_fn, ctx);

        if constexpr (std::is_same_v<T, int32_t>) {
            logsorti32(data, len, BUF_LEN);
        } else if constexpr (std::is_same_v<T, uint64_t>) {
            logsortu64(data, len, BUF_LEN);
        } else {
            printf("Not supported\n");
            return 1;
        }
    } catch (...) {
        return 1;
    }

    return 0;
}

extern "C" {
// --- i32 ---

void logsort_stable_i32(int32_t* data, size_t len) {
    logsorti32_prim(data, len, BUF_LEN);
}

uint32_t logsort_stable_i32_by(int32_t* data,
                               size_t len,
                               CompResult (*cmp_fn)(const int32_t&, const int32_t&, uint8_t*),
                               uint8_t* ctx) {
    return sort_by_impl(data, len, cmp_fn, ctx);
}

// --- u64 ---

void logsort_stable_u64(uint64_t* data, size_t len) {
    logsortu64_prim(data, len, BUF_LEN);
}

uint32_t logsort_stable_u64_by(uint64_t* data,
                               size_t len,
                               CompResult (*cmp_fn)(const uint64_t&, const uint64_t&, uint8_t*),
                               uint8_t* ctx) {
    return sort_by_impl(data, len, cmp_fn, ctx);
}

// --- ffi_string ---

void logsort_stable_ffi_string(FFIString* data, size_t len) {
    // Value would have to be sorted by indirection.
    printf("Not supported\n");
}

uint32_t logsort_stable_ffi_string_by(FFIString* data,
                                      size_t len,
                                      CompResult (*cmp_fn)(const FFIString&,
                                                           const FFIString&,
                                                           uint8_t*),
                                      uint8_t* ctx) {
    printf("Not supported\n");
    return 1;
}

// --- f128 ---

void logsort_stable_f128(F128* data, size_t len) {
    // Swaps values incorrectly, or my implementation is wrong.
    printf("Not supported\n");
}

uint32_t logsort_stable_f128_by(F128* data,
                                size_t len,
                                CompResult (*cmp_fn)(const F128&, const F128&, uint8_t*),
                                uint8_t* ctx) {
    printf("Not supported\n");
    return 1;
}

// --- 1k ---

void logsort_stable_1k(FFIOneKibiByte* data, size_t len) {
    // Value would have to be sorted by indirection.
    printf("Not supported\n");
}

uint32_t logsort_stable_1k_by(FFIOneKibiByte* data,
                              size_t len,
                              CompResult (*cmp_fn)(const FFIOneKibiByte&,
                                                   const FFIOneKibiByte&,
                                                   uint8_t*),
                              uint8_t* ctx) {
    printf("Not supported\n");
    return 1;
}
}  // extern "C"
