//===-- Data structures for sorting routines --------------------*- C++ -*-===//
//
// Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//

#ifndef LLVM_LIBC_SRC_STDLIB_QSORT_DATA_H
#define LLVM_LIBC_SRC_STDLIB_QSORT_DATA_H

#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include <cstddef>

namespace idisort {
    namespace internal {

        class ArrayGenericSize {
            std::byte* array_base;
            size_t array_len;
            size_t elem_size;

            std::byte* get_internal(size_t i) const noexcept {
                return array_base + (i * elem_size);
            }

        public:
            ArrayGenericSize(void* a, size_t s, size_t e) noexcept
                : array_base(reinterpret_cast<std::byte*>(a)), array_len(s), elem_size(e) {}

            static constexpr bool has_fixed_size() { return false; }

            void* get(size_t i) const noexcept { return reinterpret_cast<void*>(get_internal(i)); }

            void swap(size_t i, size_t j) const noexcept {
                // It's possible to use 8 byte blocks with `uint64_t`, but that
                // generates more machine code as the remainder loop gets
                // unrolled, plus 4 byte operations are more likely to be
                // efficient on a wider variety of hardware. On x86 LLVM tends
                // to unroll the block loop again into 2 16 byte swaps per
                // iteration which is another reason that 4 byte blocks yields
                // good performance even for big types.
                using block_t = uint32_t;
                constexpr size_t BLOCK_SIZE = sizeof(block_t);

                alignas(block_t) std::byte tmp_block[BLOCK_SIZE];

                std::byte* elem_i = get_internal(i);
                std::byte* elem_j = get_internal(j);

                const size_t elem_size_rem = elem_size % BLOCK_SIZE;
                const std::byte* elem_i_block_end = elem_i + (elem_size - elem_size_rem);

                while (elem_i != elem_i_block_end) {
                    memcpy(tmp_block, elem_i, BLOCK_SIZE);
                    memcpy(elem_i, elem_j, BLOCK_SIZE);
                    memcpy(elem_j, tmp_block, BLOCK_SIZE);

                    elem_i += BLOCK_SIZE;
                    elem_j += BLOCK_SIZE;
                }

                for (size_t n = 0; n < elem_size_rem; ++n) {
                    std::byte tmp = elem_i[n];
                    elem_i[n] = elem_j[n];
                    elem_j[n] = tmp;
                }
            }

            size_t len() const noexcept { return array_len; }

            // Make an Array starting at index |i| and length |s|.
            ArrayGenericSize make_array(size_t i, size_t s) const noexcept {
                return ArrayGenericSize(get_internal(i), s, elem_size);
            }

            // Reset this Array to point at a different interval of the same
            // items starting at index |i|.
            void reset_bounds(size_t i, size_t s) noexcept {
                array_base = get_internal(i);
                array_len = s;
            }
        };

        // Having a specialized Array type for sorting that knows at
        // compile-time what the size of the element is, allows for much more
        // efficient swapping and for cheaper offset calculations.
        template <size_t ELEM_SIZE>
        class ArrayFixedSize {
            std::byte* array_base;
            size_t array_len;

            std::byte* get_internal(size_t i) const noexcept {
                return array_base + (i * ELEM_SIZE);
            }

        public:
            ArrayFixedSize(void* a, size_t s) noexcept
                : array_base(reinterpret_cast<std::byte*>(a)), array_len(s) {}

            // Beware this function is used a heuristic for cheap to swap types,
            // so instantiating `ArrayFixedSize` with `ELEM_SIZE > 100` is
            // probably a bad idea perf wise.
            static constexpr bool has_fixed_size() { return true; }

            void* get(size_t i) const noexcept { return get_internal(i); }

            void swap(size_t i, size_t j) const noexcept {
                alignas(32) std::byte tmp[ELEM_SIZE];

                std::byte* elem_i = get_internal(i);
                std::byte* elem_j = get_internal(j);

                memcpy(tmp, elem_i, ELEM_SIZE);
                memmove(elem_i, elem_j, ELEM_SIZE);
                memcpy(elem_j, tmp, ELEM_SIZE);
            }

            size_t len() const noexcept { return array_len; }

            // Make an Array starting at index |i| and length |s|.
            ArrayFixedSize<ELEM_SIZE> make_array(size_t i, size_t s) const noexcept {
                return ArrayFixedSize<ELEM_SIZE>(get_internal(i), s);
            }

            // Reset this Array to point at a different interval of the same
            // items starting at index |i|.
            void reset_bounds(size_t i, size_t s) noexcept {
                array_base = get_internal(i);
                array_len = s;
            }
        };

    }  // namespace internal
}  // namespace idisort

#endif  // LLVM_LIBC_SRC_STDLIB_QSORT_DATA_H
