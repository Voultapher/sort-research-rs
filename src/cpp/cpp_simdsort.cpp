// These includes are a mess. Order is important.

#include "thirdparty/simdsort/common.h"

#include "thirdparty/simdsort/avx2-altquicksort.h"

#include <stdexcept>

#include <stdint.h>

struct CompResult;

extern "C" {
struct CompResult {
  int8_t cmp_result;
  bool is_panic;
};

// --- i32 ---

void simdsort_avx2_unstable_i32(int32_t* data, size_t len) {
  avx2_pivotonlast_sort(data, len);
}

uint32_t simdsort_avx2_unstable_i32_by(int32_t* data,
                                       size_t len,
                                       CompResult (*cmp_fn)(const int32_t&,
                                                            const int32_t&,
                                                            uint8_t*),
                                       uint8_t* ctx) {
  printf("Not supported\n");
  return 1;
}

// --- u64 ---

void simdsort_avx2_unstable_u64(uint64_t* data, size_t len) {
  printf("Not supported\n");
}

uint32_t simdsort_avx2_unstable_u64_by(uint64_t* data,
                                       size_t len,
                                       CompResult (*cmp_fn)(const uint64_t&,
                                                            const uint64_t&,
                                                            uint8_t*),
                                       uint8_t* ctx) {
  printf("Not supported\n");
  return 1;
}
}  // extern "C"
