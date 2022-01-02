#include "thirdparty/ips4o/ips4o.hpp"

#include <stdexcept>

#include <stdint.h>

#include "shared.h"

template <typename T>
uint32_t sort_by_impl(T* data,
                      size_t len,
                      CompResult (*cmp_fn)(const T&, const T&, uint8_t*),
                      uint8_t* ctx) noexcept {
  try {
    ips4o::sort(data, data + len, make_compare_fn(cmp_fn, ctx));
  } catch (...) {
    return 1;
  }

  return 0;
}

extern "C" {
// --- i32 ---

void ips4o_unstable_i32(int32_t* data, size_t len) {
  ips4o::sort(data, data + len);
}

uint32_t ips4o_unstable_i32_by(int32_t* data,
                               size_t len,
                               CompResult (*cmp_fn)(const int32_t&,
                                                    const int32_t&,
                                                    uint8_t*),
                               uint8_t* ctx) {
  return sort_by_impl(data, len, cmp_fn, ctx);
}

// --- u64 ---

void ips4o_unstable_u64(uint64_t* data, size_t len) {
  ips4o::sort(data, data + len);
}

uint32_t ips4o_unstable_u64_by(uint64_t* data,
                               size_t len,
                               CompResult (*cmp_fn)(const uint64_t&,
                                                    const uint64_t&,
                                                    uint8_t*),
                               uint8_t* ctx) {
  return sort_by_impl(data, len, cmp_fn, ctx);
}

// --- ffi_string ---

void ips4o_unstable_ffi_string(FFIString* data, size_t len) {
  ips4o::sort(reinterpret_cast<FFIStringCpp*>(data),
              reinterpret_cast<FFIStringCpp*>(data) + len);
}

uint32_t ips4o_unstable_ffi_string_by(FFIString* data,
                                      size_t len,
                                      CompResult (*cmp_fn)(const FFIString&,
                                                           const FFIString&,
                                                           uint8_t*),
                                      uint8_t* ctx) {
  return sort_by_impl(data, len, cmp_fn, ctx);
}
}  // extern "C"
