#include "thirdparty/crumsort/fluxsort.h"

#include <stdexcept>

struct CompResult;

template <typename T>
CMPFUNC* make_compare_fn(CompResult (*cmp_fn)(const T&, const T&, uint8_t*),
                         uint8_t* ctx) {
  thread_local static CompResult (*cmp_fn_local)(const T&, const T&, uint8_t*) =
      nullptr;
  thread_local static uint8_t* ctx_local = nullptr;

  cmp_fn_local = cmp_fn;
  ctx_local = ctx;

  return [](const void* a_ptr, const void* b_ptr) -> int {
    const T a = *static_cast<const T*>(a_ptr);
    const T b = *static_cast<const T*>(b_ptr);

    const auto comp_result = cmp_fn_local(a, b, ctx_local);

    if (comp_result.is_panic) {
      throw std::runtime_error{"panic in Rust comparison function"};
    }

    if (comp_result.is_less) {
      return -1;
    }
    return 1;
  };
}

template <typename T>
uint32_t sort_by_impl(T* data,
                      size_t len,
                      CompResult (*cmp_fn)(const T&, const T&, uint8_t*),
                      uint8_t* ctx) noexcept {
  try {
    fluxsort(static_cast<void*>(data), len, sizeof(T),
             make_compare_fn(cmp_fn, ctx));
  } catch (...) {
    return 1;
  }

  return 0;
}

template <typename T>
int int_cmp_func(const void* a_ptr, const void* b_ptr) {
  const T a = *static_cast<const T*>(a_ptr);
  const T b = *static_cast<const T*>(b_ptr);

  // Yeah I know everyone does a - b, but that invokes UB.
  if (a < b) {
    return -1;
  }
  return 1;
}

extern "C" {
struct CompResult {
  bool is_less;
  bool is_panic;
};

// --- i32 ---

void fluxsort_stable_i32(int32_t* data, size_t len) {
  fluxsort(static_cast<void*>(data), len, sizeof(int32_t),
           [](const void* a_ptr, const void* b_ptr) {
             return int_cmp_func<int32_t>(a_ptr, b_ptr);
           });
}

uint32_t fluxsort_stable_i32_by(int32_t* data,
                                size_t len,
                                CompResult (*cmp_fn)(const int32_t&,
                                                     const int32_t&,
                                                     uint8_t*),
                                uint8_t* ctx) {
  return sort_by_impl(data, len, cmp_fn, ctx);
}

// --- u64 ---

void fluxsort_stable_u64(uint64_t* data, size_t len) {
  fluxsort(static_cast<void*>(data), len, sizeof(uint64_t),
           [](const void* a_ptr, const void* b_ptr) {
             return int_cmp_func<uint64_t>(a_ptr, b_ptr);
           });
}

uint32_t fluxsort_stable_u64_by(uint64_t* data,
                                size_t len,
                                CompResult (*cmp_fn)(const uint64_t&,
                                                     const uint64_t&,
                                                     uint8_t*),
                                uint8_t* ctx) {
  return sort_by_impl(data, len, cmp_fn, ctx);
}
}  // extern "C"
