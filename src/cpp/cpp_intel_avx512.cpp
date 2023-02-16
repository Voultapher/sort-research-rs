#include "thirdparty/intel_avx512/avx512-32bit-qsort.hpp"
#include "thirdparty/intel_avx512/avx512-64bit-qsort.hpp"

#include <stdexcept>

#include <stdint.h>

#include "shared.h"

extern "C" {
// --- i32 ---

void intel_avx512_i32(int32_t* data, size_t len) {
  avx512_qsort(data, len);
}

uint32_t intel_avx512_i32_by(int32_t* data,
                             size_t len,
                             CompResult (*cmp_fn)(const int32_t&,
                                                  const int32_t&,
                                                  uint8_t*),
                             uint8_t* ctx) {
  printf("Not supported\n");
  return 1;
}

// --- u64 ---

void intel_avx512_u64(uint64_t* data, size_t len) {
  printf("Not supported\n");
}

uint32_t intel_avx512_u64_by(uint64_t* data,
                             size_t len,
                             CompResult (*cmp_fn)(const uint64_t&,
                                                  const uint64_t&,
                                                  uint8_t*),
                             uint8_t* ctx) {
  printf("Not supported\n");
  return 1;
}

// --- ffi_string ---

void intel_avx512_ffi_string(FFIString* data, size_t len) {
  printf("Not supported\n");
}

uint32_t intel_avx512_ffi_string_by(FFIString* data,
                                    size_t len,
                                    CompResult (*cmp_fn)(const FFIString&,
                                                         const FFIString&,
                                                         uint8_t*),
                                    uint8_t* ctx) {
  printf("Not supported\n");
  return 1;
}

// --- f128 ---

void intel_avx512_f128(F128* data, size_t len) {
  printf("Not supported\n");
}

uint32_t intel_avx512_f128_by(F128* data,
                              size_t len,
                              CompResult (*cmp_fn)(const F128&,
                                                   const F128&,
                                                   uint8_t*),
                              uint8_t* ctx) {
  printf("Not supported\n");
  return 1;
}

// --- 1k ---

void intel_avx512_1k(F128* data, size_t len) {
  printf("Not supported\n");
}

uint32_t intel_avx512_1k_by(F128* data,
                            size_t len,
                            CompResult (*cmp_fn)(const F128&,
                                                 const F128&,
                                                 uint8_t*),
                            uint8_t* ctx) {
  printf("Not supported\n");
  return 1;
}
}  // extern "C"
