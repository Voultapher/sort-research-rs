//===-- Implementation header for qsort utilities ---------------*- C++ -*-===//
//
// Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//

#ifndef LLVM_LIBC_SRC_STDLIB_QSORT_UTIL_H
#define LLVM_LIBC_SRC_STDLIB_QSORT_UTIL_H

#include "heap_sort.h"
#include "qsort_data.h"
#include "quick_sort.h"

#define LIBC_QSORT_QUICK_SORT 1
#define LIBC_QSORT_HEAP_SORT 2

#ifdef LIBC_OPTIMIZE_FOR_SIZE
#define LIBC_QSORT_IMPL LIBC_QSORT_HEAP_SORT
#else
#ifndef LIBC_QSORT_IMPL
#define LIBC_QSORT_IMPL LIBC_QSORT_QUICK_SORT
#endif  // LIBC_QSORT_IMPL
#endif  // LIBC_OPTIMIZE_FOR_SIZE

#if (LIBC_QSORT_IMPL != LIBC_QSORT_QUICK_SORT && LIBC_QSORT_IMPL != LIBC_QSORT_HEAP_SORT)
#error "LIBC_QSORT_IMPL is not recognized."
#endif

#if defined(__GNUC__)
#define ___INLINE_ALWAYS __attribute__((always_inline))
#define ___INLINE_NEVER __attribute__((noinline))
#else
#define ___INLINE_ALWAYS inline
#define ___INLINE_NEVER
#endif

namespace idisort {
    namespace internal {

        template <typename F>
        void unstable_sort(void* array, size_t array_len, size_t elem_size, const F& is_less) {
            if (array == nullptr || array_len == 0 || elem_size == 0)
                return;

#if LIBC_QSORT_IMPL == LIBC_QSORT_QUICK_SORT
            switch (elem_size) {
            case 4: {
                auto arr_fixed_size = internal::ArrayFixedSize<4>(array, array_len);
                quick_sort(arr_fixed_size, is_less);
                return;
            }
            case 8: {
                auto arr_fixed_size = internal::ArrayFixedSize<8>(array, array_len);
                quick_sort(arr_fixed_size, is_less);
                return;
            }
            case 16: {
                auto arr_fixed_size = internal::ArrayFixedSize<16>(array, array_len);
                quick_sort(arr_fixed_size, is_less);
                return;
            }
            default:
                auto arr_generic_size = internal::ArrayGenericSize(array, array_len, elem_size);
                quick_sort(arr_generic_size, is_less);
                return;
            }
#elif LIBC_QSORT_IMPL == LIBC_QSORT_HEAP_SORT
            auto arr_generic_size = internal::ArrayGenericSize(array, array_len, elem_size);
            heap_sort(arr_generic_size, is_less);
#endif
        }

    }  // namespace internal
}  // namespace idisort

#endif  // LLVM_LIBC_SRC_STDLIB_QSORT_UTIL_H
