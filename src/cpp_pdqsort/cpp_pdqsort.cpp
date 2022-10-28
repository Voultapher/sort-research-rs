#include "pdqsort.h"

template <typename T>
auto make_compare_fn(bool (*cmp_fn)(const T&, const T&, uint8_t*),
                     uint8_t* ctx) {
  return [cmp_fn, ctx](const T& a, const T& b) mutable -> bool {
    return cmp_fn(a, b, ctx);
  };
}

extern "C" {
// --- i32 ---

void pdqsort_unstable_i32(int32_t* data, size_t len) {
  pdqsort(data, data + len);
}

void pdqsort_unstable_i32_by(int32_t* data,
                             size_t len,
                             bool (*cmp_fn)(const int32_t&,
                                            const int32_t&,
                                            uint8_t*),
                             uint8_t* ctx) {
  pdqsort(data, data + len, make_compare_fn(cmp_fn, ctx));
}

// --- u64 ---

void pdqsort_unstable_u64(uint64_t* data, size_t len) {
  pdqsort(data, data + len);
}

void pdqsort_unstable_u64_by(uint64_t* data,
                             size_t len,
                             bool (*cmp_fn)(const uint64_t&,
                                            const uint64_t&,
                                            uint8_t*),
                             uint8_t* ctx) {
  pdqsort(data, data + len, make_compare_fn(cmp_fn, ctx));
}
}  // extern "C"
