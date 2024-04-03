// These includes are a mess. Order is important.

#include "thirdparty/simdsort/common.h"

#include "thirdparty/simdsort/avx2-altquicksort.h"

#include <stdexcept>

#include <stdint.h>

#include "shared.h"

extern "C" {
// --- i32 ---

void simdsort_avx2_i32(int32_t* data, size_t len) {
  avx2_pivotonlast_sort(data, len);
}

uint32_t simdsort_avx2_i32_by(int32_t* data,
                              size_t len,
                              CompResult (*cmp_fn)(const int32_t&,
                                                   const int32_t&,
                                                   uint8_t*),
                              uint8_t* ctx) {
  printf("Not supported\n");
  return 1;
}

// --- u64 ---

void simdsort_avx2_u64(uint64_t* data, size_t len) {
  printf("Not supported\n");
}

uint32_t simdsort_avx2_u64_by(uint64_t* data,
                              size_t len,
                              CompResult (*cmp_fn)(const uint64_t&,
                                                   const uint64_t&,
                                                   uint8_t*),
                              uint8_t* ctx) {
  printf("Not supported\n");
  return 1;
}

// --- ffi_string ---

void simdsort_avx2_ffi_string(FFIString* data, size_t len) {
  printf("Not supported\n");
}

uint32_t simdsort_avx2_ffi_string_by(FFIString* data,
                                     size_t len,
                                     CompResult (*cmp_fn)(const FFIString&,
                                                          const FFIString&,
                                                          uint8_t*),
                                     uint8_t* ctx) {
  printf("Not supported\n");
  return 1;
}

// --- f128 ---

void simdsort_avx2_f128(F128* data, size_t len) {
  printf("Not supported\n");
}

uint32_t simdsort_avx2_f128_by(F128* data,
                               size_t len,
                               CompResult (*cmp_fn)(const F128&,
                                                    const F128&,
                                                    uint8_t*),
                               uint8_t* ctx) {
  printf("Not supported\n");
  return 1;
}

// --- 1k ---

void simdsort_avx2_1k(FFIOneKibiByte* data, size_t len) {
  printf("Not supported\n");
}

uint32_t simdsort_avx2_1k_by(FFIOneKibiByte* data,
                             size_t len,
                             CompResult (*cmp_fn)(const FFIOneKibiByte&,
                                                  const FFIOneKibiByte&,
                                                  uint8_t*),
                             uint8_t* ctx) {
  printf("Not supported\n");
  return 1;
}
}  // extern "C"
