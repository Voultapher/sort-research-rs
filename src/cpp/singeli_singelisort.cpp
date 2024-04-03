#include "thirdparty/singelisort/sort.c"

#include <stdexcept>
#include <vector>

#include <stdint.h>

#include "shared.h"

size_t aux_alloc_size(size_t len) {
  return len + 4 * (len < 1 << 16 ? len : 1 << 16);
}

extern "C" {
// --- i32 ---

void singelisort_i32(int32_t* data, size_t len) {
  std::vector<int32_t> aux_memory{};
  aux_memory.reserve(aux_alloc_size(len));
  sort32(data, static_cast<uint64_t>(len), aux_memory.data(),
         aux_memory.capacity() * sizeof(int32_t));
}

uint32_t singelisort_i32_by(int32_t* data,
                            size_t len,
                            CompResult (*cmp_fn)(const int32_t&,
                                                 const int32_t&,
                                                 uint8_t*),
                            uint8_t* ctx) {
  printf("Not supported\n");
  return 1;
}

// --- u64 ---

void singelisort_u64(uint64_t* data, size_t len) {
  std::vector<uint64_t> aux_memory{};
  aux_memory.reserve(aux_alloc_size(len));
  sort_u64(data, static_cast<uint64_t>(len), aux_memory.data(),
           aux_memory.capacity() * sizeof(uint64_t));
}

uint32_t singelisort_u64_by(uint64_t* data,
                            size_t len,
                            CompResult (*cmp_fn)(const uint64_t&,
                                                 const uint64_t&,
                                                 uint8_t*),
                            uint8_t* ctx) {
  printf("Not supported\n");
  return 1;
}

// --- ffi_string ---

void singelisort_ffi_string(FFIString* data, size_t len) {
  printf("Not supported\n");
}

uint32_t singelisort_ffi_string_by(FFIString* data,
                                   size_t len,
                                   CompResult (*cmp_fn)(const FFIString&,
                                                        const FFIString&,
                                                        uint8_t*),
                                   uint8_t* ctx) {
  printf("Not supported\n");
  return 1;
}

// --- f128 ---

void singelisort_f128(F128* data, size_t len) {
  printf("Not supported\n");
}

uint32_t singelisort_f128_by(F128* data,
                             size_t len,
                             CompResult (*cmp_fn)(const F128&,
                                                  const F128&,
                                                  uint8_t*),
                             uint8_t* ctx) {
  printf("Not supported\n");
  return 1;
}

// --- 1k ---

void singelisort_1k(FFIOneKibiByte* data, size_t len) {
  printf("Not supported\n");
}

uint32_t singelisort_1k_by(FFIOneKibiByte* data,
                           size_t len,
                           CompResult (*cmp_fn)(const FFIOneKibiByte&,
                                                const FFIOneKibiByte&,
                                                uint8_t*),
                           uint8_t* ctx) {
  printf("Not supported\n");
  return 1;
}
}  // extern "C"
