#include "thirdparty/blockquicksort/blocked_double_pivot_check_mosqrt.h"

#include <stdint.h>
#include <stdexcept>

#include "shared.h"

template <typename T>
uint32_t sort_by_impl(T* data,
                      size_t len,
                      CompResult (*cmp_fn)(const T&, const T&, uint8_t*),
                      uint8_t* ctx) noexcept {
  // BlockQuicksort does not provide a way to specify a custom comparator
  // function, so we have to wrap it inside a type with custom comparison
  // function.
  CompWrapper<T>::cmp_fn_local = cmp_fn;
  CompWrapper<T>::ctx_local = ctx;

  try {
    blocked_double_pivot_check_mosqrt::sort(
        reinterpret_cast<CompWrapper<T>*>(data),
        reinterpret_cast<CompWrapper<T>*>(data) + len,
        std::less<CompWrapper<T>>{});
  } catch (...) {
    return 1;
  }

  return 0;
}

extern "C" {
// --- i32 ---

void blockquicksort_unstable_i32(int32_t* data, size_t len) {
  blocked_double_pivot_check_mosqrt::sort(data, data + len,
                                          std::less<int32_t>{});
}

uint32_t blockquicksort_unstable_i32_by(int32_t* data,
                                        size_t len,
                                        CompResult (*cmp_fn)(const int32_t&,
                                                             const int32_t&,
                                                             uint8_t*),
                                        uint8_t* ctx) {
  return sort_by_impl(data, len, cmp_fn, ctx);
}

// --- u64 ---

void blockquicksort_unstable_u64(uint64_t* data, size_t len) {
  blocked_double_pivot_check_mosqrt::sort(data, data + len,
                                          std::less<uint64_t>{});
}

uint32_t blockquicksort_unstable_u64_by(uint64_t* data,
                                        size_t len,
                                        CompResult (*cmp_fn)(const uint64_t&,
                                                             const uint64_t&,
                                                             uint8_t*),
                                        uint8_t* ctx) {
  return sort_by_impl(data, len, cmp_fn, ctx);
}

// --- ffi_string ---

void blockquicksort_unstable_ffi_string(FFIString* data, size_t len) {
  blocked_double_pivot_check_mosqrt::sort(
      reinterpret_cast<FFIStringCpp*>(data),
      reinterpret_cast<FFIStringCpp*>(data) + len, std::less<FFIStringCpp>{});
}

uint32_t blockquicksort_unstable_ffi_string_by(
    FFIString* data,
    size_t len,
    CompResult (*cmp_fn)(const FFIString&, const FFIString&, uint8_t*),
    uint8_t* ctx) {
  return sort_by_impl(data, len, cmp_fn, ctx);
}

// --- f128 ---

void blockquicksort_unstable_f128(F128* data, size_t len) {
  blocked_double_pivot_check_mosqrt::sort(
      reinterpret_cast<F128Cpp*>(data), reinterpret_cast<F128Cpp*>(data) + len,
      std::less<F128Cpp>{});
}

uint32_t blockquicksort_unstable_f128_by(F128* data,
                                         size_t len,
                                         CompResult (*cmp_fn)(const F128&,
                                                              const F128&,
                                                              uint8_t*),
                                         uint8_t* ctx) {
  return sort_by_impl(data, len, cmp_fn, ctx);
}

// --- 1k ---

void blockquicksort_unstable_1k(FFIOneKiloByte* data, size_t len) {
  blocked_double_pivot_check_mosqrt::sort(
      reinterpret_cast<FFIOneKiloByteCpp*>(data),
      reinterpret_cast<FFIOneKiloByteCpp*>(data) + len,
      std::less<FFIOneKiloByteCpp>{});
}

uint32_t blockquicksort_unstable_1k_by(
    FFIOneKiloByte* data,
    size_t len,
    CompResult (*cmp_fn)(const FFIOneKiloByte&,
                         const FFIOneKiloByte&,
                         uint8_t*),
    uint8_t* ctx) {
  return sort_by_impl(data, len, cmp_fn, ctx);
}
}  // extern "C"
