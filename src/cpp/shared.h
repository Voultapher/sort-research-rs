#pragma once

#include <stddef.h>

extern "C" {
struct CompResult {
  int8_t cmp_result;
  bool is_panic;
};

struct FFIString {
  char* data;
  size_t len;
  size_t capacity;
};
}

#ifdef __cplusplus
#include <string_view>

// This should have the same layout as FFIString so that it can be
// reinterpret_cast.
struct FFIStringCpp : public FFIString {
  std::string_view as_str() const noexcept {
    return std::string_view{data, len};
  }

  bool operator<(const FFIStringCpp& other) const noexcept {
    return as_str() < other.as_str();
  }
  bool operator<=(const FFIStringCpp& other) const noexcept {
    return as_str() <= other.as_str();
  }
  bool operator>(const FFIStringCpp& other) const noexcept {
    return as_str() > other.as_str();
  }
  bool operator>=(const FFIStringCpp& other) const noexcept {
    return as_str() > other.as_str();
  }
  bool operator==(const FFIStringCpp& other) const noexcept {
    return as_str() == other.as_str();
  }
};

template <typename T>
auto make_compare_fn(CompResult (*cmp_fn)(const T&, const T&, uint8_t*),
                     uint8_t* ctx) {
  return [cmp_fn, ctx](const T& a, const T& b) mutable -> bool {
    const auto comp_result = cmp_fn(a, b, ctx);

    if (comp_result.is_panic) {
      throw std::runtime_error{"panic in Rust comparison function"};
    }

    return comp_result.cmp_result == -1;
  };
}

typedef int CMPFUNC(const void* a, const void* b);

template <typename T>
CMPFUNC* make_compare_fn_c(CompResult (*cmp_fn)(const T&, const T&, uint8_t*),
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

    return comp_result.cmp_result;
  };
}

#endif
