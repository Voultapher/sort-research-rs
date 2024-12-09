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

namespace LIBC_NAMESPACE_DECL {
    void qsort(void* array,
               size_t array_size,
               size_t elem_size,
               int (*compare)(const void*, const void*)) {
        if (array == nullptr || array_size == 0 || elem_size == 0)
            return;
        internal::Comparator c(compare);

        auto arr = internal::Array(reinterpret_cast<uint8_t*>(array), array_size, elem_size, c);

        internal::sort(arr);
    }
}

#endif  // LLVM_LIBC_SRC_STDLIB_QSORT_H
