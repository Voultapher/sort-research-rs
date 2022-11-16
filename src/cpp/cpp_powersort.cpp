#include "thirdparty/powersort/powersort.h"
#include "thirdparty/powersort/powersort_4way.h"

#include <compare>
#include <stdexcept>
#include <vector>

#include <stdint.h>

struct CompResult;

template <typename T>
using vec_iter = std::vector<T>::iterator;

template <typename T>
using powersort = algorithms::powersort<
    /*Iterator=*/T,
    /*minRunLen=*/24,
    /*mergingMethod*/ algorithms::merging_methods::COPY_BOTH,
    /*onlyIncreasingRuns=*/false,
    /*nodePowerImplementation=*/algorithms::MOST_SIGNIFICANT_SET_BIT,
    /*usePowerIndexedStack=*/false>;

template <typename T>
using powersort_4way = algorithms::powersort_4way<
    /*Iterator=*/T,
    /*minRunLen=*/24,
    // For faster perf use WILLEM_TUNED but this can't sort slices with custom
    // types anymore, and it can't correctly sort slices that contain the
    // sentinel. GENERAL_BY_STAGES works without sentinel requirement.
    /*mergingMethod*/ algorithms::merging4way_methods::GENERAL_BY_STAGES,
    /*onlyIncreasingRuns=*/false,
    /*nodePowerImplementation=*/algorithms::MOST_SIGNIFICANT_SET_BIT4,
    /*useParallelArraysForStack=*/false,
    /*useCheckFirstMergeLoop=*/true,
    /*useSpecialized3wayMerge=*/true>;

template <typename T, template <typename> class SortT>
uint32_t sort_by_impl(T* data,
                      size_t len,
                      CompResult (*cmp_fn)(const T&, const T&, uint8_t*),
                      uint8_t* ctx) noexcept {
  try {
    thread_local static CompResult (*cmp_fn_local)(const T&, const T&,
                                                   uint8_t*) = nullptr;
    thread_local static uint8_t* ctx_local = nullptr;

    cmp_fn_local = cmp_fn;
    ctx_local = ctx;

    // Powersort does not provide a way to specify a custom comparator function,
    // so we have to wrap it inside a type with custom comparison function.
    struct CompWrapper {
      std::strong_ordering operator<=>(const CompWrapper& other) {
        const auto comp_result = cmp_fn_local(_value, other._value, ctx_local);

        if (comp_result.is_panic) {
          throw std::runtime_error{"panic in Rust comparison function"};
        }

        switch (comp_result.cmp_result) {
          case -1:
            return std::strong_ordering::less;
          case 0:
            return std::strong_ordering::equal;
          case 1:
            return std::strong_ordering::greater;
          default:
            throw std::runtime_error{"Unknown cmp_result value"};
        }
      }

      T _value;
    };

    using iter_t = vec_iter<CompWrapper>;
    // Let's just pray they are layout equivalent.
    SortT<iter_t>{}.sort(iter_t{reinterpret_cast<CompWrapper*>(data)},
                         iter_t{reinterpret_cast<CompWrapper*>(data + len)});
  } catch (const std::exception& exc) {
    // fprintf(stderr, "[ERROR]: %s\n", exc.what());
    return 1;
  } catch (...) {
    return 1;
  }

  return 0;
}

extern "C" {
struct CompResult {
  int8_t cmp_result;
  bool is_panic;
};

// --- i32 ---

void powersort_stable_i32(int32_t* data, size_t len) {
  // Uses default configuration.
  using iter_t = vec_iter<int32_t>;
  powersort<iter_t>{}.sort(iter_t{data}, iter_t{data + len});
}

uint32_t powersort_stable_i32_by(int32_t* data,
                                 size_t len,
                                 CompResult (*cmp_fn)(const int32_t&,
                                                      const int32_t&,
                                                      uint8_t*),
                                 uint8_t* ctx) {
  return sort_by_impl<int32_t, powersort>(data, len, cmp_fn, ctx);
}

// --- u64 ---

void powersort_stable_u64(uint64_t* data, size_t len) {
  // Uses default configuration.
  using iter_t = vec_iter<uint64_t>;
  powersort<iter_t>{}.sort(iter_t{data}, iter_t{data + len});
}

uint32_t powersort_stable_u64_by(uint64_t* data,
                                 size_t len,
                                 CompResult (*cmp_fn)(const uint64_t&,
                                                      const uint64_t&,
                                                      uint8_t*),
                                 uint8_t* ctx) {
  return sort_by_impl<uint64_t, powersort>(data, len, cmp_fn, ctx);
}

// --- 4 way merging ---

// --- i32 ---

void powersort_4way_stable_i32(int32_t* data, size_t len) {
  // Uses default configuration.
  using iter_t = vec_iter<int32_t>;
  powersort_4way<iter_t>{}.sort(iter_t{data}, iter_t{data + len});
}

uint32_t powersort_4way_stable_i32_by(int32_t* data,
                                      size_t len,
                                      CompResult (*cmp_fn)(const int32_t&,
                                                           const int32_t&,
                                                           uint8_t*),
                                      uint8_t* ctx) {
  return sort_by_impl<int32_t, powersort_4way>(data, len, cmp_fn, ctx);
}

// --- u64 ---

void powersort_4way_stable_u64(uint64_t* data, size_t len) {
  // Uses default configuration.
  using iter_t = vec_iter<uint64_t>;
  powersort_4way<iter_t>{}.sort(iter_t{data}, iter_t{data + len});
}

uint32_t powersort_4way_stable_u64_by(uint64_t* data,
                                      size_t len,
                                      CompResult (*cmp_fn)(const uint64_t&,
                                                           const uint64_t&,
                                                           uint8_t*),
                                      uint8_t* ctx) {
  return sort_by_impl<uint64_t, powersort_4way>(data, len, cmp_fn, ctx);
}

}  // extern "C"
