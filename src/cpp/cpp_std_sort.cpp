#include <algorithm>
#include <stdexcept>

#include <stdint.h>

#include "shared.h"

template <typename T, typename F>
uint32_t sort_stable_by_impl(T* data, size_t len, F cmp_fn, uint8_t* ctx) noexcept {
    try {
        std::stable_sort(data, data + len, make_compare_fn<T>(cmp_fn, ctx));
    } catch (...) {
        return 1;
    }

    return 0;
}

template <typename T, typename F>
uint32_t sort_unstable_by_impl(T* data, size_t len, F cmp_fn, uint8_t* ctx) noexcept {
    try {
        std::sort(data, data + len, make_compare_fn<T>(cmp_fn, ctx));
    } catch (...) {
        return 1;
    }

    return 0;
}

#if defined(STD_LIB_SYS)
#define MAKE_FUNC_NAME(name, suffix) name##_sys_##suffix
#elif defined(STD_LIB_LIBCXX)
#define MAKE_FUNC_NAME(name, suffix) name##_libcxx_##suffix
#endif

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
    std::stable_sort(reinterpret_cast<FFIStringCpp*>(data),
                     reinterpret_cast<FFIStringCpp*>(data) + len);
}

uint32_t MAKE_FUNC_NAME(sort_stable, ffi_string_by)(FFIString* data,
                                                    size_t len,
                                                    CompResult (*cmp_fn)(const FFIString&,
                                                                         const FFIString&,
                                                                         uint8_t*),
                                                    uint8_t* ctx) {
    return sort_stable_by_impl(reinterpret_cast<FFIStringCpp*>(data), len, cmp_fn, ctx);
}

void MAKE_FUNC_NAME(sort_unstable, ffi_string)(FFIString* data, size_t len) {
    std::sort(reinterpret_cast<FFIStringCpp*>(data), reinterpret_cast<FFIStringCpp*>(data) + len);
}

uint32_t MAKE_FUNC_NAME(sort_unstable, ffi_string_by)(FFIString* data,
                                                      size_t len,
                                                      CompResult (*cmp_fn)(const FFIString&,
                                                                           const FFIString&,
                                                                           uint8_t*),
                                                      uint8_t* ctx) {
    return sort_unstable_by_impl(reinterpret_cast<FFIStringCpp*>(data), len, cmp_fn, ctx);
}

// --- f128 ---

void MAKE_FUNC_NAME(sort_stable, f128)(F128* data, size_t len) {
    std::stable_sort(reinterpret_cast<F128Cpp*>(data), reinterpret_cast<F128Cpp*>(data) + len);
}

uint32_t MAKE_FUNC_NAME(sort_stable,
                        f128_by)(F128* data,
                                 size_t len,
                                 CompResult (*cmp_fn)(const F128&, const F128&, uint8_t*),
                                 uint8_t* ctx) {
    return sort_stable_by_impl(reinterpret_cast<F128Cpp*>(data), len, cmp_fn, ctx);
}

void MAKE_FUNC_NAME(sort_unstable, f128)(F128* data, size_t len) {
    std::sort(reinterpret_cast<F128Cpp*>(data), reinterpret_cast<F128Cpp*>(data) + len);
}

uint32_t MAKE_FUNC_NAME(sort_unstable,
                        f128_by)(F128* data,
                                 size_t len,
                                 CompResult (*cmp_fn)(const F128&, const F128&, uint8_t*),
                                 uint8_t* ctx) {
    return sort_unstable_by_impl(reinterpret_cast<F128Cpp*>(data), len, cmp_fn, ctx);
}

// --- 1k ---

void MAKE_FUNC_NAME(sort_stable, 1k)(FFIOneKibiByte* data, size_t len) {
    std::stable_sort(reinterpret_cast<FFIOneKibiByteCpp*>(data),
                     reinterpret_cast<FFIOneKibiByteCpp*>(data) + len);
}

uint32_t MAKE_FUNC_NAME(sort_stable, 1k_by)(FFIOneKibiByte* data,
                                            size_t len,
                                            CompResult (*cmp_fn)(const FFIOneKibiByte&,
                                                                 const FFIOneKibiByte&,
                                                                 uint8_t*),
                                            uint8_t* ctx) {
    return sort_stable_by_impl(reinterpret_cast<FFIOneKibiByteCpp*>(data), len, cmp_fn, ctx);
}

void MAKE_FUNC_NAME(sort_unstable, 1k)(FFIOneKibiByte* data, size_t len) {
    std::sort(reinterpret_cast<FFIOneKibiByteCpp*>(data),
              reinterpret_cast<FFIOneKibiByteCpp*>(data) + len);
}

uint32_t MAKE_FUNC_NAME(sort_unstable, 1k_by)(FFIOneKibiByte* data,
                                              size_t len,
                                              CompResult (*cmp_fn)(const FFIOneKibiByte&,
                                                                   const FFIOneKibiByte&,
                                                                   uint8_t*),
                                              uint8_t* ctx) {
    return sort_unstable_by_impl(reinterpret_cast<FFIOneKibiByteCpp*>(data), len, cmp_fn, ctx);
}
}  // extern "C"
