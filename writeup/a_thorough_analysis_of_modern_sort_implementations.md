# A thorough analysis of modern sort implementations. And introduction of novel instruction-parallel sort implementations.

WIP: Please do not publish.

Author: Lukas Bergdoll @Voultapher  
Date: TODO

Bias disclaimer. The author of this analysis is the author of the ipn family of sort implementations.

### Introduction

Implementing a sort operation with the help of computers goes back to the early 1950s. The problem statement is deceptively simple. Take a list of elements, that implement a total order and use a comparison function to swap elements until it's sorted. Now, 70 years later new more resource-efficient ways to implement this operation are still being discovered. It's an active field of study in science, pdqsort 2016 [1](https://arxiv.org/pdf/2106.05123.pdf), ips4o [2](https://arxiv.org/pdf/1705.02257.pdf), multiway powersort 2022 [3](https://arxiv.org/pdf/2209.06909.pdf), glidesort 2023 [4](TODO), and many more. There are various directions science is exploiting, efficient sort implementations, running single threaded on modern superscalar, out-of-order and speculative CPUs. Efficient implementations running on multiple such threads, implementations running on massively parallel in-order GPUs. Exploration of better best-case, average-case and worst-case runtime. Exploiting existing patterns in the input data. Exploration of different characteristics, stable/unstable in-place/allocating and more. This analysis will focus on single threaded (ST), stable and unstable sort implementations, fit for system programming standard libraries. In many ways there exists a gulf between science and industry. Most scientific papers focus solely on sorting signed 32-bit integers. Leaving several, if not the most complicated parts, required for a robust general purpose sort implementation, unexplored and unanswered. This analysis will shine light into these properties and more.

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
- cpp_ips4o_unstable         | https://github.com/ips4o/ips4o
- cpp_blockquicksort         | https://github.com/weissan/BlockQuicksort
- c_crumsort_unstable        | https://github.com/scandum/crumsort
```

#### Other

```
- rust_radsort               | radix sort https://github.com/jakubvaltar/radsort
- cpp_simdsort               | avx2-altquicksort supports only i32 https://github.com/WojciechMula/simd-sort
- cpp_highway                | vectorized quicksort https://github.com/google/highway/tree/master/hwy/contrib/sort
```

### Property analysis

The table below, shows a summary of analyzed properties:

Properties:
- Functional. Does the implementation successfully pass the test suite of different input patterns and supported types?
- Generic. Does the implementation support arbitrary user defined types?
- Stack. Max stack array size.
- Heap. Max heap allocation size.
- Ord safety. What happens if the user defined type or comparison function does not implement a total order. Eg. in C++ your comparison function does `[](const auto& a, const auto& b) { return a.x <= b.x; }`? O == unspecified order but original elements, E == exception/panic and unspecified order but original elements, H == infinite loop, C == crash, eg. heap-buffer-overflow (UB).
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
| cpp_ips4o_unstable         | ✅         | ✅      | ?      | -     | C          | 🚫               | 1. 🚫 2. 🚫     | -    |
| cpp_blockquicksort         | ✅         | ✅      | ?      | -     | C          | 🚫               | 1. ✅ 2. 🚫     | -    |
| c_crumsort_unstable        | ✅         | ⚠️ (4)  | 512    | -     | C          | 🚫 (8)           | 1. 🚫 2. 🚫     | -    |
| rust_radsort               | ✅         | 🚫      | ?      | N     | -          | -                | -               | -    |
| cpp_simdsort_unstable      | ✅         | 🚫      | ?      | -     | -          | -                | -               | -    | 
| cpp_vqsort_unstable   | ✅         | 🚫      | ?      | -     | -          | -                | -               | -    | 

Footnotes:
1. cpp_powersort_stable uses `vector::resize` for it's buffer, requiring that `T` is default constructible.
2. cpp_powersort_4way_stable uses `vector::resize` for it's buffer, requiring that `T` is default constructible.cpp_powersort_4way_stable offers many configuration options, one of them is mergingMethod. GENERAL_BY_STAGES supports all user-defined types, but is relatively slow. WILLEM_TUNED is faster but requires a sentinel value. Ie. `u64::MAX` but this can't correctly sort slices that contain the sentinel, making it unsuitable for a general purpose sort.
3. c_fluxsort_stable uses auxiliary stack and heap memory that may be under-aligned for types with alignment larger than fundamental alignment. The sort interface requires either, very large performance sacrifices or source level modification.
4. c_crumsort_unstable uses auxiliary stack memory that may be under-aligned for types with alignment larger than fundamental alignment. The sort interface requires either, very large performance sacrifices or source level modification.
5. rust_ipn_stable will only use a 32 element stack array if `T` is at most 4 times the size of a pointer, limiting the upper end stack usage. Otherwise falling back to only using a single stack element.
6. rust_ipn_stable will try to allocate a buffer of size of N and fall back to using a buffer of size N/2 if the allocation fails. Incurring some slowdown.
7. rust_ipn_unstable will only use a 40 element stack array if `T` is at most 4 times the size of a pointer, limiting the upper end stack usage. Otherwise falling back to only using a single stack element.
8. c_fluxsort_stable and c_crumsort_unstable are developed as C based sorts. C has no concept of exceptions, or stack unwinding. So this property would only be relevant if compiled as C++ code.

### Failure modes

One goal of this analysis is to investigate what is required for a robust general purpose sort implementation. Seemingly-safe usage can lead to Undefined Behavior (UB). Especially for Rust based sort implementations, that expose a notionally safe-to-use interface. However, all of the properties that make the implementation more resilient against misuse, can also be achieved in C++ code. C++, a language proven impossible to develop in a memory safe fashion [8](https://alexgaynor.net/2020/may/27/science-on-memory-unsafety-and-security/) without restricting it to a small subset, would be a prime candidate to limit failure modes, by making their standard library sort implementation more robust. As demonstrated with the ipn family of sort implementations, even the strictest of requirements can be achieved while simultaneously greatly improving efficiency.

Below will be examples in C++ and Rust, demonstrating user-code that could trigger UB if the property is not given.

#### Ord safety

Only O and E mean that the implementation safely handles such cases. Especially in C++ it's trivial and presumably a common mistake.

```cpp
sort(data.begin(), data.end(), [](const auto& a, const auto& b) {
    return a <= b; // correct would be a < b.
});
```

The standard sort interface in Rust avoids most of these problems, but they are still certainly possible.

```rust
data.sort_by(|a, b| {
    if a == b {
        return Ordering::Less;
    }

    a.cmp(b)
});
```

Notably, the only C and C++ implementations that avoid UB in such scenario are, cpp_std_libcxx_stable and the powersort family of implementations. Especially with how easy it is to do this mistake in C++ it appears negligent that all major standard library implementations fail in that regard. In contrast the ipn family of implementations improves usability by raising a recoverable and deterministic panic, informing the user of logic bug.

#### Exception safety

C++ and Rust are both languages with scope based destructors (RAII), and stack unwinding. Together they prove a powerful abstraction for manual memory management. At the same time, they can make implementing generic code more complex. Every single point in the sort implementation that calls the user provided comparison function, must assume that the call may return via exception/panic and will unwind the stack. Usually a reliable way to deal with this, is to use scope-guards that will return the input into a valid state, ie. set of original elements with all possible modifications applied to them.

```cpp
sort(data.begin(), data.end(), [](const auto& a, const auto& b) {
    if (some_condition) {
        throw std::runtime_error{"unwind"};
    }

    return a < b;
});
```

```rust
data.sort_by(|a, b| {
    if some_condition {
        panic!("unwind");
    }

    a.cmp(b)
});
```

Again a clear distinction appears, every single Rust based implementation upholds this property, even for simple integers. While every single C++ based implementation fails in this regard. And while it may seem tempting to switch to some slower implementation for types that are `std::is_trivially_destructible`/`!core::mem::needs_drop`, doing so would still risk problems for types such as raw pointers [9](https://godbolt.org/z/rWe9rn9G3). They are trivially destructible, yet a user may use them to sort by indirection. And a sane assumption would be that, no-matter how the sort completes, either returning normally or via exception/panic that every original pointer is still in the array. Yet, across all tested C++ implementation some pointers may be duplicated, quickly leading to UB such as use-after-free (double-free). Arguably even special casing for builtin integer types could lead to problems, in the presence of a user-defined comparison function. These integers may be interpreted as pointers, even though such code has pointer-provenance issues [10](https://faultlore.com/blah/fix-rust-pointers/), such code is currently a common occurrence in real world code.

### Observable comp

Both C++ and Rust offer ways to mutate a value through a const/shared reference. C++ achieves this with the help of the mutable type specifier, while Rust builds safe-to-use abstractions on top of the language builtin `UnsafeCell`. As a consequence of this it's possible to observe every call to the user-provided comparison function as a stack value modification. However, as soon as auxiliary memory, be it stack or heap, is used, unsafe bitwise duplications of the object need to be performed. If such a duplicated element is used as input to the user-provided comparison function, it may be modified in a way that must be observed when the sort completes, either 1. by returning normally or 2. by raising an exception/panic. A benign scenario with surprising consequences would be counting the comparisons performed by incrementing a counter held within the elements inside each call to the user-provided comparison function. If the property of guaranteed observable comparison is not met, the result may be wildly inaccurate in describing how many times the user-provided comparison function has been called. A more problematic scenario invoking UB would be, a user-defined type holding a pointer that is conditionally freed and set to null as part of the user-provided comparison function. If this modification is not observed after the sort completes, code that relies on a null-pointer check to see if it was already freed may run into use-after-free UB.

```cpp
struct ValWithPtr {
    int32_t val;
    mutable std::string* ptr;
};

sort(data, data + len,  [&some_condition](const auto& a, const auto& b) {
    if (some_condition) {
        free(a.ptr);
        a.ptr = nullptr;
        free(b.ptr);
        b.ptr = nullptr;
    }

    return a.val < b.val;    
});
```

```rust
pub struct ValWithPtr {
    val: i32,
    ptr: Cell<Option<Box<str>>>,
}

pub fn x(data: &mut [ValWithPtr]) {
    data.sort_by(|a, b| {
        if some_condition {
            a.ptr.set(None);
            b.ptr.set(None);
        }

        a.val.cmp(&b.val)
    });
}
```

The C language has no such constructs that would allow safe modifications through a const/shared references, as such the C based sort implementations understandably fail this property. Assuming the sort completes by returning normally, every C++ and Rust based implementation upholds this property with the notable exception being cpp_ips4o_unstable. If the sort complete via exception/panic not even every Rust implementation upholds this property. The Rust code required to trigger use-after-free is completely safe and as such making rust_dmsort_unstable unsound [11](https://github.com/emilk/drop-merge-sort/issues/23).
