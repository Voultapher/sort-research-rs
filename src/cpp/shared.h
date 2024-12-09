#pragma once

#include <stddef.h>
#include <stdint.h>

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

struct FFIOneKibiByte {
    int64_t values[128];
};
}

#if __cplusplus >= 201703L
#include <string_view>

// This should have the same layout as FFIString so that it can be
// reinterpret_cast.
struct FFIStringCpp : public FFIString {
    std::string_view as_str() const noexcept { return std::string_view{data, len}; }

// Disable the define if you want a version of FFIStringCpp that behaves like
// a trivially copyable type. Can impact behavior and performance.
#define FFI_STRING_AS_SEMANTIC_CPP_TYPE

#if defined(FFI_STRING_AS_SEMANTIC_CPP_TYPE) && !defined(SORT_INCOMPATIBLE_WITH_SEMANTIC_CPP_TYPE)

    FFIStringCpp(const FFIStringCpp&) = delete;
    FFIStringCpp(FFIStringCpp&& other) {
        data = other.data;
        len = other.len;
        capacity = other.capacity;

        other.data = nullptr;
    }

    FFIStringCpp& operator=(const FFIStringCpp&) = delete;
    FFIStringCpp& operator=(FFIStringCpp&& other) {
        if (this != &other) {
            data = other.data;
            len = other.len;
            capacity = other.capacity;

            other.data = nullptr;
        }

        return *this;
    }

    ~FFIStringCpp() {
        // C++ is allowed to destroy moved from FFIStringCpp values, but those
        // should have set data to nullptr.
        if (data) {
            // This should really never be called from C++ code. Rust owns the data
            // and the C++ code only every should have access to a pointer underlying
            // the Rust owned slice. free is the wrong function to call, the right one
            // would be the Rust allocator that was used to build the Rust side
            // FFIString. It serves only code-gen reasons to make a comparison fairer.
            free(data);
        }
    }
#endif  // FFI_STRING_AS_SEMANTIC_CPP_TYPE

    bool operator<(const FFIStringCpp& other) const noexcept { return as_str() < other.as_str(); }
    bool operator<=(const FFIStringCpp& other) const noexcept { return as_str() <= other.as_str(); }
    bool operator>(const FFIStringCpp& other) const noexcept { return as_str() > other.as_str(); }
    bool operator>=(const FFIStringCpp& other) const noexcept { return as_str() >= other.as_str(); }
    bool operator==(const FFIStringCpp& other) const noexcept { return as_str() == other.as_str(); }
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

struct FFIOneKibiByteCpp : public FFIOneKibiByte {
    int64_t as_i64() const noexcept { return values[11] + values[55] + values[77]; }

    bool operator<(const FFIOneKibiByteCpp& other) const noexcept {
        return as_i64() < other.as_i64();
    }
    bool operator<=(const FFIOneKibiByteCpp& other) const noexcept {
        return as_i64() <= other.as_i64();
    }
    bool operator>(const FFIOneKibiByteCpp& other) const noexcept {
        return as_i64() > other.as_i64();
    }
    bool operator>=(const FFIOneKibiByteCpp& other) const noexcept {
        return as_i64() >= other.as_i64();
    }
    bool operator==(const FFIOneKibiByteCpp& other) const noexcept {
        return as_i64() == other.as_i64();
    }
};

template <typename T, typename F>
struct CompWrapper {
    // Not a big fan of this approach, but it works.
    thread_local static inline F cmp_fn_local;
    thread_local static inline uint8_t* ctx_local;

    std::strong_ordering operator<=>(const CompWrapper& other) const {
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

    T _value;  // Let's just pray it has the same layout as T.
};

template <typename T, typename F>
auto make_compare_fn(F cmp_fn, uint8_t* ctx) {
    return [cmp_fn, ctx](const T& a, const T& b) mutable -> bool {
        const auto comp_result = cmp_fn(a, b, ctx);

        if (comp_result.is_panic) {
            throw std::runtime_error{"panic in Rust comparison function"};
        }

        return comp_result.cmp_result == -1;
    };
}

// --- C ---

typedef int CMPFUNC(const void* a, const void* b);

template <typename T>
CMPFUNC* make_compare_fn_c(CompResult (*cmp_fn)(const T&, const T&, uint8_t*), uint8_t* ctx) {
    thread_local static CompResult (*cmp_fn_local)(const T&, const T&, uint8_t*) = nullptr;
    thread_local static uint8_t* ctx_local = nullptr;

    cmp_fn_local = cmp_fn;
    ctx_local = ctx;

    return [](const void* a_ptr, const void* b_ptr) -> int {
        const T& a = *static_cast<const T*>(a_ptr);
        const T& b = *static_cast<const T*>(b_ptr);

        const auto comp_result = cmp_fn_local(a, b, ctx_local);

        if (comp_result.is_panic) {
            throw std::runtime_error{"panic in Rust comparison function"};
        }

        return comp_result.cmp_result;
    };
}

template <typename T>
int int_cmp_func(const void* a_ptr, const void* b_ptr) {
    const T& a = *static_cast<const T*>(a_ptr);
    const T& b = *static_cast<const T*>(b_ptr);

    // Yeah I know everyone does a - b, but that invokes UB.
    //
    // if (a < b) {
    //   return -1;
    // } else if (a > b) {
    //   return 1;
    // }
    // return 0;
    //
    // Alternative branchless version, that optimizes particularly well with
    // clang. gcc sees a 2x speedup for random inputs with this.
    // https://godbolt.org/z/EfYxd7rqP

    const bool is_less = a < b;
    const bool is_more = a > b;
    return (static_cast<int>(is_less) * -1) + (static_cast<int>(is_more) * 1);
}

#endif
