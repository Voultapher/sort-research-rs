#include <stdexcept>

#include <stdint.h>

#include "shared.h"

template <typename T>
using cmp_fn_ptr_t = int64_t (*)(T, T);

// Communicate to the go side that a panic happened.
constexpr int64_t PANIC_MAGIC_NUMBER = 777;

template <typename T>
cmp_fn_ptr_t<T> make_compare_fn_go(CompResult (*cmp_fn)(const T&, const T&, uint8_t*),
                                   uint8_t* ctx) {
    thread_local static CompResult (*cmp_fn_local)(const T&, const T&, uint8_t*) = nullptr;
    thread_local static uint8_t* ctx_local = nullptr;

    cmp_fn_local = cmp_fn;
    ctx_local = ctx;

    return [](T a, T b) -> int64_t {
        const auto comp_result = cmp_fn_local(a, b, ctx_local);

        if (comp_result.is_panic) {
            return PANIC_MAGIC_NUMBER;
        }

        return comp_result.cmp_result;
    };
}

#define NOT_IMPL(STABILITY, TYPE_NAME, TYPE)                                              \
    void golang_std_##STABILITY##_##TYPE_NAME(TYPE* data, size_t len) {                   \
        printf("Not supported\n");                                                        \
    }                                                                                     \
                                                                                          \
    uint32_t golang_std_##STABILITY##_##TYPE_NAME##_by(                                   \
        TYPE* data, size_t len, CompResult (*cmp_fn)(const TYPE&, const TYPE&, uint8_t*), \
        uint8_t* ctx) {                                                                   \
        printf("Not supported\n");                                                        \
        return 1;                                                                         \
    }

#define IMPL(STABILITY, TYPE_NAME, TYPE, SORT_NAME_BASE)                                       \
    void golang_std_##STABILITY##_##TYPE_NAME(TYPE* data, size_t len) {                        \
        SORT_NAME_BASE(GoSlice{/*data:*/ reinterpret_cast<void*>(data),                        \
                               /*len:*/ static_cast<GoInt>(len),                               \
                               /*cap:*/ static_cast<GoInt>(len)});                             \
    }                                                                                          \
                                                                                               \
    uint32_t golang_std_##STABILITY##_##TYPE_NAME##_by(                                        \
        TYPE* data, size_t len, CompResult (*cmp_fn)(const TYPE&, const TYPE&, uint8_t*),      \
        uint8_t* ctx) {                                                                        \
        const auto did_panic = SORT_NAME_BASE##By(                                             \
            GoSlice{/*data:*/ reinterpret_cast<void*>(data), /*len:*/ static_cast<GoInt>(len), \
                    /*cap:*/ static_cast<GoInt>(len)},                                         \
            make_compare_fn_go(cmp_fn, ctx));                                                  \
                                                                                               \
        if (did_panic) {                                                                       \
            return 1;                                                                          \
        }                                                                                      \
                                                                                               \
        return 0;                                                                              \
    }

extern "C" {
#include <golang_std_ffi_lib.h>

int64_t i32_by_bridge(cmp_fn_ptr_t<int32_t> fn_ptr, int32_t a, int32_t b) {
    return fn_ptr(a, b);
}

int64_t u64_by_bridge(cmp_fn_ptr_t<uint64_t> fn_ptr, uint64_t a, uint64_t b) {
    return fn_ptr(a, b);
}

IMPL(unstable, i32, int32_t, UnstableSortI32);
IMPL(unstable, u64, uint64_t, UnstableSortU64);
NOT_IMPL(unstable, ffi_string, FFIString);
NOT_IMPL(unstable, f128, F128);
NOT_IMPL(unstable, 1k, FFIOneKibiByte);

IMPL(stable, i32, int32_t, StableSortI32);
IMPL(stable, u64, uint64_t, StableSortU64);
NOT_IMPL(stable, ffi_string, FFIString);
NOT_IMPL(stable, f128, F128);
NOT_IMPL(stable, 1k, FFIOneKibiByte);

}  // extern "C"
