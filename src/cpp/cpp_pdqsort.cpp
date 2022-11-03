#include "pdqsort.h"

#include <stdexcept>

struct CompResult;

template <typename T>
auto make_compare_fn(CompResult (*cmp_fn)(const T&, const T&, uint8_t*),
                     uint8_t* ctx) {
  return [cmp_fn, ctx](const T& a, const T& b) mutable -> bool {
    const auto comp_result = cmp_fn(a, b, ctx);

    if (comp_result.is_panic) {
      throw std::runtime_error{"panic in Rust comparison function"};
    }

    return comp_result.is_less;
  };
}

template <typename T>
uint32_t sort_by_impl(T* data,
                      size_t len,
                      CompResult (*cmp_fn)(const T&, const T&, uint8_t*),
                      uint8_t* ctx) noexcept {
  try {
    pdqsort(data, data + len, make_compare_fn(cmp_fn, ctx));
  } catch (...) {
    return 1;
  }

  return 0;
}

extern "C" {
struct CompResult {
  bool is_less;
  bool is_panic;
};

// --- i32 ---

void pdqsort_unstable_i32(int32_t* data, size_t len) {
  pdqsort(data, data + len);
}

uint32_t pdqsort_unstable_i32_by(int32_t* data,
                                 size_t len,
                                 CompResult (*cmp_fn)(const int32_t&,
                                                      const int32_t&,
                                                      uint8_t*),
                                 uint8_t* ctx) {
  return sort_by_impl(data, len, cmp_fn, ctx);
}

// --- u64 ---

void pdqsort_unstable_u64(uint64_t* data, size_t len) {
  pdqsort(data, data + len);
}

uint32_t pdqsort_unstable_u64_by(uint64_t* data,
                                 size_t len,
                                 CompResult (*cmp_fn)(const uint64_t&,
                                                      const uint64_t&,
                                                      uint8_t*),
                                 uint8_t* ctx) {
  return sort_by_impl(data, len, cmp_fn, ctx);
}
}  // extern "C"
