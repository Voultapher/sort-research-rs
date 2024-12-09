//===-- Implementation header for qsort -------------------------*- C++ -*-===//
//
// Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//

#ifndef LLVM_LIBC_SRC_STDLIB_QSORT_H
#define LLVM_LIBC_SRC_STDLIB_QSORT_H

#include "qsort_util.h"

#include "stddef.h"

namespace idisort {
    // Never inline to mimic hidden implementation in .cpp in real lib.
    ___INLINE_NEVER void qsort(void* array,
                               size_t array_size,
                               size_t elem_size,
                               int (*compare)(const void*, const void*)) {
        internal::unstable_sort(
            array, array_size, elem_size,
            [compare](const void* a, const void* b) noexcept -> bool { return compare(a, b) < 0; });
    }
}

#endif  // LLVM_LIBC_SRC_STDLIB_QSORT_H
