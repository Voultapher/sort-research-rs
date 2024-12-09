// Re-using cpp_std_sort would be nice but there is too much C++11 or newer
// stuff involved to make that work easily.
// So limit this to integers.

#include <algorithm>
#include <stdexcept>

#include <stdint.h>

#include "shared.h"

template <typename T>
struct CompareLambda {
    CompareLambda(CompResult (*i_cmp_fn)(const T&, const T&, uint8_t*), uint8_t* i_ctx)
        : cmp_fn(i_cmp_fn), ctx(i_ctx) {}

    bool operator()(const T& a, const T& b) {
        const CompResult comp_result = cmp_fn(a, b, ctx);

        if (comp_result.is_panic) {
            throw std::runtime_error("panic in Rust comparison function");
        }

        return comp_result.cmp_result == -1;
    }

    CompResult (*cmp_fn)(const T&, const T&, uint8_t*);
    uint8_t* ctx;
};

template <typename T>
uint32_t sort_stable_by_impl(T* data,
                             size_t len,
                             CompResult (*cmp_fn)(const T&, const T&, uint8_t*),
                             uint8_t* ctx) {
    try {
        std::stable_sort(data, data + len, CompareLambda<T>(cmp_fn, ctx));
    } catch (...) {
        return 1;
    }

    return 0;
}

template <typename T>
uint32_t sort_unstable_by_impl(T* data,
                               size_t len,
                               CompResult (*cmp_fn)(const T&, const T&, uint8_t*),
                               uint8_t* ctx) {
    try {
        std::sort(data, data + len, CompareLambda<T>(cmp_fn, ctx));
    } catch (...) {
        return 1;
    }

    return 0;
}

#define MAKE_FUNC_NAME(name, suffix) name##_gcc4_3_##suffix

extern "C" {
// --- i32 ---

void MAKE_FUNC_NAME(sort_stable, i32)(int32_t* data, size_t len) {
    std::stable_sort(data, data + len);
}

uint32_t MAKE_FUNC_NAME(sort_stable,
                        i32_by)(int32_t* data,
                                size_t len,
                                CompResult (*cmp_fn)(const int32_t&, const int32_t&, uint8_t*),
                                uint8_t* ctx) {
    return sort_stable_by_impl(data, len, cmp_fn, ctx);
}

void MAKE_FUNC_NAME(sort_unstable, i32)(int32_t* data, size_t len) {
    std::sort(data, data + len);
}

uint32_t MAKE_FUNC_NAME(sort_unstable,
                        i32_by)(int32_t* data,
                                size_t len,
                                CompResult (*cmp_fn)(const int32_t&, const int32_t&, uint8_t*),
                                uint8_t* ctx) {
    return sort_unstable_by_impl(data, len, cmp_fn, ctx);
}

// --- u64 ---

void MAKE_FUNC_NAME(sort_stable, u64)(uint64_t* data, size_t len) {
    std::stable_sort(data, data + len);
}

uint32_t MAKE_FUNC_NAME(sort_stable,
                        u64_by)(uint64_t* data,
                                size_t len,
                                CompResult (*cmp_fn)(const uint64_t&, const uint64_t&, uint8_t*),
                                uint8_t* ctx) {
    return sort_stable_by_impl(data, len, cmp_fn, ctx);
}

void MAKE_FUNC_NAME(sort_unstable, u64)(uint64_t* data, size_t len) {
    std::sort(data, data + len);
}

uint32_t MAKE_FUNC_NAME(sort_unstable,
                        u64_by)(uint64_t* data,
                                size_t len,
                                CompResult (*cmp_fn)(const uint64_t&, const uint64_t&, uint8_t*),
                                uint8_t* ctx) {
    return sort_unstable_by_impl(data, len, cmp_fn, ctx);
}

// --- FFIString ---

void MAKE_FUNC_NAME(sort_stable, ffi_string)(FFIString* data, size_t len) {
    printf("Not supported\n");
}

uint32_t MAKE_FUNC_NAME(sort_stable, ffi_string_by)(FFIString* data,
                                                    size_t len,
                                                    CompResult (*cmp_fn)(const FFIString&,
                                                                         const FFIString&,
                                                                         uint8_t*),
                                                    uint8_t* ctx) {
    printf("Not supported\n");
    return 1;
}

void MAKE_FUNC_NAME(sort_unstable, ffi_string)(FFIString* data, size_t len) {
    printf("Not supported\n");
}

uint32_t MAKE_FUNC_NAME(sort_unstable, ffi_string_by)(FFIString* data,
                                                      size_t len,
                                                      CompResult (*cmp_fn)(const FFIString&,
                                                                           const FFIString&,
                                                                           uint8_t*),
                                                      uint8_t* ctx) {
    printf("Not supported\n");
    return 1;
}

// --- f128 ---

void MAKE_FUNC_NAME(sort_stable, f128)(F128* data, size_t len) {
    printf("Not supported\n");
}

uint32_t MAKE_FUNC_NAME(sort_stable,
                        f128_by)(F128* data,
                                 size_t len,
                                 CompResult (*cmp_fn)(const F128&, const F128&, uint8_t*),
                                 uint8_t* ctx) {
    printf("Not supported\n");
    return 1;
}

void MAKE_FUNC_NAME(sort_unstable, f128)(F128* data, size_t len) {
    printf("Not supported\n");
}

uint32_t MAKE_FUNC_NAME(sort_unstable,
                        f128_by)(F128* data,
                                 size_t len,
                                 CompResult (*cmp_fn)(const F128&, const F128&, uint8_t*),
                                 uint8_t* ctx) {
    printf("Not supported\n");
    return 1;
}

// --- 1k ---

void MAKE_FUNC_NAME(sort_stable, 1k)(FFIOneKibiByte* data, size_t len) {
    printf("Not supported\n");
}

uint32_t MAKE_FUNC_NAME(sort_stable, 1k_by)(FFIOneKibiByte* data,
                                            size_t len,
                                            CompResult (*cmp_fn)(const FFIOneKibiByte&,
                                                                 const FFIOneKibiByte&,
                                                                 uint8_t*),
                                            uint8_t* ctx) {
    printf("Not supported\n");
    return 1;
}

void MAKE_FUNC_NAME(sort_unstable, 1k)(FFIOneKibiByte* data, size_t len) {
    printf("Not supported\n");
}

uint32_t MAKE_FUNC_NAME(sort_unstable, 1k_by)(FFIOneKibiByte* data,
                                              size_t len,
                                              CompResult (*cmp_fn)(const FFIOneKibiByte&,
                                                                   const FFIOneKibiByte&,
                                                                   uint8_t*),
                                              uint8_t* ctx) {
    printf("Not supported\n");
    return 1;
}
}  // extern "C"
