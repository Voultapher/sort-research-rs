#include <algorithm>
#include <stdexcept>

#include <stdint.h>

#include "shared.h"

template <typename T>
uint32_t sort_stable_by_impl(T* data,
                             size_t len,
                             CompResult (*cmp_fn)(const T&, const T&, uint8_t*),
                             uint8_t* ctx) noexcept {
  try {
    std::stable_sort(data, data + len, make_compare_fn(cmp_fn, ctx));
  } catch (...) {
    return 1;
  }

  return 0;
}

template <typename T>
uint32_t sort_unstable_by_impl(T* data,
                               size_t len,
                               CompResult (*cmp_fn)(const T&,
                                                    const T&,
                                                    uint8_t*),
                               uint8_t* ctx) noexcept {
  try {
    std::sort(data, data + len, make_compare_fn(cmp_fn, ctx));
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

uint32_t MAKE_FUNC_NAME(sort_stable, i32_by)(
    int32_t* data,
    size_t len,
    CompResult (*cmp_fn)(const int32_t&, const int32_t&, uint8_t*),
    uint8_t* ctx) {
  return sort_stable_by_impl(data, len, cmp_fn, ctx);
}

void MAKE_FUNC_NAME(sort_unstable, i32)(int32_t* data, size_t len) {
  std::sort(data, data + len);
}

uint32_t MAKE_FUNC_NAME(sort_unstable, i32_by)(
    int32_t* data,
    size_t len,
    CompResult (*cmp_fn)(const int32_t&, const int32_t&, uint8_t*),
    uint8_t* ctx) {
  return sort_unstable_by_impl(data, len, cmp_fn, ctx);
}

// --- u64 ---

void MAKE_FUNC_NAME(sort_stable, u64)(uint64_t* data, size_t len) {
  std::stable_sort(data, data + len);
}

uint32_t MAKE_FUNC_NAME(sort_stable, u64_by)(
    uint64_t* data,
    size_t len,
    CompResult (*cmp_fn)(const uint64_t&, const uint64_t&, uint8_t*),
    uint8_t* ctx) {
  return sort_stable_by_impl(data, len, cmp_fn, ctx);
}

void MAKE_FUNC_NAME(sort_unstable, u64)(uint64_t* data, size_t len) {
  std::sort(data, data + len);
}

uint32_t MAKE_FUNC_NAME(sort_unstable, u64_by)(
    uint64_t* data,
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

uint32_t MAKE_FUNC_NAME(sort_stable, ffi_string_by)(
    FFIString* data,
    size_t len,
    CompResult (*cmp_fn)(const FFIString&, const FFIString&, uint8_t*),
    uint8_t* ctx) {
  return sort_stable_by_impl(data, len, cmp_fn, ctx);
}

void MAKE_FUNC_NAME(sort_unstable, ffi_string)(FFIString* data, size_t len) {
  std::sort(reinterpret_cast<FFIStringCpp*>(data),
            reinterpret_cast<FFIStringCpp*>(data) + len);
}

uint32_t MAKE_FUNC_NAME(sort_unstable, ffi_string_by)(
    FFIString* data,
    size_t len,
    CompResult (*cmp_fn)(const FFIString&, const FFIString&, uint8_t*),
    uint8_t* ctx) {
  return sort_unstable_by_impl(data, len, cmp_fn, ctx);
}
}  // extern "C"
