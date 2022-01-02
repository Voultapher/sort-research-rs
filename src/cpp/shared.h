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

struct F128 {
  double x;
  double y;
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
    return as_str() >= other.as_str();
  }
  bool operator==(const FFIStringCpp& other) const noexcept {
    return as_str() == other.as_str();
  }
};

struct F128Cpp : public F128 {
  double as_div_val() const noexcept { return x / y; }

  bool operator<(const F128Cpp& other) const noexcept {
    return as_div_val() < other.as_div_val();
  }
  bool operator<=(const F128Cpp& other) const noexcept {
    return as_div_val() <= other.as_div_val();
  }
  bool operator>(const F128Cpp& other) const noexcept {
    return as_div_val() > other.as_div_val();
  }
  bool operator>=(const F128Cpp& other) const noexcept {
    return as_div_val() >= other.as_div_val();
  }
  bool operator==(const F128Cpp& other) const noexcept {
    return as_div_val() == other.as_div_val();
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

template <typename T>
int int_cmp_func(const void* a_ptr, const void* b_ptr) {
  const T a = *static_cast<const T*>(a_ptr);
  const T b = *static_cast<const T*>(b_ptr);

  // Yeah I know everyone does a - b, but that invokes UB.
  // if (a < b) {
  //   return -1;
  // } else if (a > b) {
  //   return 1;
  // }
  // return 0;

  // This optimizes particularly well with clang.
  // gcc sees a 2x speedup for random inputs with this.
  // https://godbolt.org/z/ETdbYoMTK

  // Alternative branchless version.
  const bool is_less = a < b;
  const bool is_more = a > b;
  return (is_less * -1) + (is_more * 1);
}

// This is broken, crumsort and fluxsort break the individual F128 values.
//
// static constexpr bool F128_SUPPORT = sizeof(F128) == sizeof(long double) &&
//                                      alignof(F128) <= alignof(max_align_t);

// int f128_c_cmp_func(const void* a_ptr, const void* b_ptr) {
//   const F128Cpp a = *static_cast<const F128Cpp*>(a_ptr);
//   const F128Cpp b = *static_cast<const F128Cpp*>(b_ptr);

//   printf("a.x: %f, a.y: %f\n", a.x, a.y);
//   printf("b.x: %f, b.y: %f\n", b.x, b.y);
//   const int is_less = a < b;
//   printf("Is less: %d\n", is_less);

//   if (a < b) {
//     return -1;
//   } else if (a > b) {
//     return 1;
//   }
//   return 0;
// }

#endif
