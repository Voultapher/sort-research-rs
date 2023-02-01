# A thorough analysis of modern sort implementations. And introduction of novel instruction-parallel sort implementations.

Implementing a sort operation with the help of computers goes back to the early 1950s. The problem statement is deceptively simple. Take a list of elements, that implement a total order and use a comparison function to swap elements until it's sorted. Now, 70 years later we are still finding new more resource-efficient ways to implement this operation. It's an active field of study in science, pdqsort 2016 [1](https://arxiv.org/pdf/2106.05123.pdf), ips4o [2](https://arxiv.org/pdf/1705.02257.pdf), multiway powersort 2022 [3](https://arxiv.org/pdf/2209.06909.pdf), glidesort 2023 [4](TODO), and many more. The are various directions science is exploiting, efficient sort implementations, running single threaded on modern superscalar, out-of-order and speculative CPUs. Efficient implementations running on multiple such threads, implementations running on massively parallel in-order GPUs. Exploration of better best-case, average-case and worst-case runtime. Exploiting existing patterns in the input data. Exploration of different characteristics, stable/unstable in-place/allocating and more. This analysis will focus on single threaded (ST), stable and unstable sort implementations, fit for system programming standard libraries. In many ways there exists a gulf between science and industry. Most scientific papers focus solely on sorting signed 32-bit integers. Leaving several, if not the most complicated parts, required for a robust general purpose sort implementation, unexplored and unanswered. This analysis will shine light into these properties and more.

The words sort implementation and sort algorithm, are expressly *not* used interchangeably. Practically all modern implementations are hybrids, using multiple sort algorithms. As such, the words 'sort algorithm' will only be used to talk about the algorithmic nature of specific building blocks.

Common beliefs about sorting implementations entail: TODO I don't like this part too much.

- Quicksort is in-place
- Quicksort is unstable
- Stable sorts with n log(n) worst case require auxiliary memory
- Efficiency improvements are limited to non-random data

This analysis will show that most of these beliefs are false. Quicksort can be made stable by using auxiliary memory. There are stable sort algorithms that use *no* auxiliary memory, like insertion sort. However insertion sort has a runtime that grows quadratically with the input size. However there are algorithmically efficient stable sorts that use a fixed size buffer ie. 512 and no further auxiliary memory eg. Block sort, also known as WikiSort [5](https://raw.githubusercontent.com/BonzaiThePenguin/WikiSort/master/tamc2008.pdf). The introduction of robust general purpose sort implementations ipn_stable sort and ipn_unstable sort, demonstrate a runtime improvement on modern commodity hardware for random unsigned 64 bit integers, compared to the best available standard library implementations, ~2x for ipn_unstable and ~3x for ipn_stable. And ~1.5x and ~1x respectively, compared to the best implementations available. Conversely, these speedups also signify improvements in energy efficiency, ie. yielding a ~2x reduction in energy consumed to sort such an input.

Benchmarking is notoriously tricky, and especially synthetic benchmarks may not be representative. An incomplete list of relevant factors:

- Input size
- Input type (price to move? price to compare?)
- Input pattern (already sorted, random, cardinality, streaks, mixed etc.)
- Hardware prediction and cache effects

A general purpose sort will try to be good in many situations, while not being too bad in the rest. On a wide selection of commodity hardware, including small embedded chips, up to large server chips. That's no easy feat.

This analysis is based on research performed over the course of most of 2022 and the start of 2023. Inspired by other work in this space [6](https://danlark.org/2022/04/20/changing-stdsort-at-googles-scale-and-beyond/) [7](https://github.com/scandum/fluxsort), it tries to shine light on the inner workings of modern high-performance sort implementations. The implementations that were analyzed, source vendored ca. mid to late 2022:

#### Stable

```
- rust_std_stable            | `slice::sort` https://github.com/rust-lang/rust
- rust_ipn_stable            | https://github.com/Voultapher/sort-research-rs
- rust_wpwoodjr_stable       | https://github.com/wpwoodjr/rust-1
- cpp_std_gnu_stable         | libstdc++ `std::stable_sort` gcc 12.2
- cpp_std_libcxx_stable      | libc++ `std::stable_sort` clang 15.0
- cpp_std_msvc_stable        | TODO
- cpp_powersort_stable       | https://github.com/sebawild/powersort
- cpp_powersort_4way_stable  | https://github.com/sebawild/powersort
- c_fluxsort_stable          | https://github.com/scandum/fluxsort
```

#### Unstable

```
- rust_std_unstable          | `slice::sort_unstable` https://github.com/rust-lang/rust
- rust_ipn_unstable          | https://github.com/Voultapher/sort-research-rs
- rust_dmsort_unstable       | https://github.com/emilk/drop-merge-sort
- cpp_std_gnu_unstable       | libstdc++ `std::sort` gcc 12.2
- cpp_std_libcxx_unstable    | libc++ `std::sort` clang 15.0
- cpp_std_msvc_unstable      | TODO
- cpp_pdqsort_unstable       | https://github.com/orlp/pdqsort
- cpp_simdsort_unstable      | avx2-altquicksort supports only i32 https://github.com/WojciechMula/simd-sort
- cpp_ips4o_unstable         | https://github.com/ips4o/ips4o
- cpp_blockquicksort         | https://github.com/weissan/BlockQuicksort
- c_crumsort_unstable        | https://github.com/scandum/crumsort
```

#### Other

```
- rust_radsort               | radix sort https://github.com/jakubvaltar/radsort
```

The table below, shows a summary of analyzed properties:

Properties:
- Functional. Does the implementation successfully pass the test suite of different input patterns and supported types?
- Generic. Does the implementation support arbitrary user defined types?
- Stack. Max stack array size.
- Heap. Max heap allocation size.
- Ord safety. What happens if the user defined type or comparison function does not implement a total order. Eg. in C++ your comparison function does `[](const auto& a, const auto& b) { return a.x <= b.x; }`? O == unspecified order but original elements, E == ecxception/panic and unspecified order but original elements, H == infinite loop, C == crash, eg. heap-buffer-overflow (UB).
- Exception safety. What happens, if the user provided comparison function throws an exception/panic? ✅ means it retains the original input set in an unspecified order, 🚫 means it may have duplicated elements in the input.
- Observable comp. If the type has interior mutability, will every modification incurred by calling the user defined comparison function with const/shared references be visible in the input, after the sort function returns 1. normally 2. panic. If exception safety is not given, it is practically impossible to achieve 2. here.
- Miri. Does the test-suite succeed if run under [miri](https://github.com/rust-lang/miri)?

| Name                       | Functional | Generic | Stack  | Heap  | Ord safety | Exception safety | Observable comp | Miri |
|----------------------------|------------|---------|--------|------ |------------|------------------|-----------------|------|
| rust_std_stable            | ✅         | ✅      | 1      | N/2   | O          | ✅               | 1. ✅ 2. ✅     | ✅   |
| rust_ipn_stable            | ✅         | ✅      | 32 (5) | N (6) | O or E     | ✅               | 1. ✅ 2. ✅     | ✅   |
| rust_wpwoodjr_stable       | ✅         | ✅      | 1      | N/2   | O          | ✅               | 1. ✅ 2. ✅     | 🚫   |
| cpp_std_gnu_stable         | ✅         | ✅      | ?      | ?     | C          | 🚫               | 1. ✅ 2. 🚫     | -    |
| cpp_std_libcxx_stable      | ✅         | ✅      | ?      | ?     | O          | 🚫               | 1. ✅ 2. 🚫     | -    |
| cpp_std_msvc_stable        | ✅         | ✅      | ?      | ?     | C          | 🚫               | 1. ✅ 2. 🚫     | -    |
| cpp_powersort_stable       | ✅         | ⚠️ (1)  | ?      | N     | O          | 🚫               | 1. ✅ 2. 🚫     | -    |
| cpp_powersort_4way_stable  | ✅         | ⚠️ (2)  | ?      | N     | O          | 🚫               | 1. ✅ 2. 🚫     | -    |
| c_fluxsort_stable          | ✅         | ⚠️ (3)  | 32     | N     | C          | 🚫 (8)           | 1. 🚫 2. 🚫     | -    |
| rust_std_unstable          | ✅         | ✅      | 1      | -     | O          | ✅               | 1. ✅ 2. ✅     | ✅   |
| rust_ipn_unstable          | ✅         | ✅      | 40 (7) | -     | O or E     | ✅               | 1. ✅ 2. ✅     | ✅   |
| rust_dmsort_unstable       | ✅         | ✅      | ?      | -     | O          | ✅               | 1. ✅ 2. 🚫     | 🚫   |
| cpp_std_gnu_unstable       | ✅         | ✅      | ?      | -     | C          | 🚫               | 1. ✅ 2. 🚫     | -    |
| cpp_std_libcxx_unstable    | ✅         | ✅      | ?      | -     | H          | 🚫               | 1. ✅ 2. 🚫     | -    |
| cpp_std_msvc_unstable      | ✅         | ✅      | ?      | -     | C          | 🚫               | 1. ✅ 2. 🚫     | -    |
| cpp_pdqsort_unstable       | ✅         | ✅      | 1      | -     | H or C     | 🚫               | 1. ✅ 2. 🚫     | -    |
| cpp_simdsort_unstable      | ✅         | 🚫      | ?      | -     | -          | -                | -               | -    | 
| cpp_ips4o_unstable         | ✅         | ✅      | ?      | -     | C          | 🚫               | 1. 🚫 2. 🚫     | -    |
| cpp_blockquicksort         | ✅         | ✅      | ?      | -     | C          | 🚫               | 1. ✅ 2. 🚫     | -    |
| c_crumsort_unstable        | ✅         | ⚠️ (4)  | 512    | -     | C          | 🚫 (8)           | 1. 🚫 2. 🚫     | -    |
| rust_radsort               | ✅         | 🚫      | ?      | N     | -          | -                | -               | -    |

Footnotes:
1. cpp_powersort_stable uses `vector::resize` for it's buffer, requiring that `T` is default constructible.
2. cpp_powersort_4way_stable uses `vector::resize` for it's buffer, requiring that `T` is default constructible.cpp_powersort_4way_stable offers many configuration options, one of them is mergingMethod. GENERAL_BY_STAGES supports all user-defined types, but is relatively slow. WILLEM_TUNED is faster but requires a sentinel value. Ie. `u64::MAX` but this can't correctly sort slices that contain the sentinel, making it unsuitable for a general purpose sort.
3. c_fluxsort_stable uses auxiliary stack and heap memory that may be under-aligned for types with alignment larger than fundamental alignment. The sort interface requires either, very large performance sacrifices or source level modification.
4. c_crumsort_unstable uses auxiliary stack memory that may be under-aligned for types with alignment larger than fundamental alignment. The sort interface requires either, very large performance sacrifices or source level modification.
5. rust_ipn_stable will only use a 32 element stack array if `T` is at most 4 times the size of a pointer, limiting the upper end stack usage. Otherwise falling back to only using a single stack element.
6. rust_ipn_stable will try to allocate a buffer of size of N and fall back to using a buffer of size N/2 if the allocation fails. Incurring some slowdown.
7. rust_ipn_unstable will only use a 40 element stack array if `T` is at most 4 times the size of a pointer, limiting the upper end stack usage. Otherwise falling back to only using a single stack element.
8. c_fluxsort_stable and c_crumsort_unstable are developed as C based sorts. C has no concept of exceptions, or stack unwinding. So this property would only be relevant if compiled as C++ code.
