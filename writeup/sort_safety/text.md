# A categorization of memory safety for various sort implementations

Author: Lukas Bergdoll @Voultapher  
Date: TODO (DD-MM-YYYY)

This is an analysis of sort implementations and their ability, or lack thereof, to avoid undefined behavior (UB) under various usage scenarios.

TL;DR: The combination of complex generic implementations striving for performance with arbitrary logic in user-defined comparison functions, makes generic high-performance sort implementations particularly difficult to implement in a way that avoids UB under every usage scenario. Even a sort implementation using only memory-safe abstractions might not be enough to guarantee UB free adjacent logic.

---

Bias disclaimer. The author of this analysis is the author of ipnsort.


## Introduction

Implementing a sort operation with the help of computers goes back to the early 1950s. The problem statement is deceptively simple. Take a list of elements and use a comparison function that implements a [strict weak ordering](https://en.wikipedia.org/wiki/Weak_ordering#Strict_weak_orderings) to swap elements until it's sorted. Now, 70 years later new and more resource-efficient ways to implement this operation are still being discovered. It's an active field of study in science, [BlockQuicksort](https://arxiv.org/pdf/1604.06697.pdf) 2016, [ips4o](https://arxiv.org/pdf/1705.02257.pdf) 2017, [pdqsort](https://arxiv.org/pdf/2106.05123.pdf) 2021, [Multiway Powersort](https://arxiv.org/pdf/2209.06909.pdf) 2022, and many more. There are various directions science is exploring. Efficient sort implementations running single threaded on modern superscalar, out-of-order and speculative CPUs. Efficient implementations running on multiple such threads. Implementations running on massively parallel in-order GPUs. Exploration of better best-case, average-case and worst-case runtime. Exploiting existing patterns in the input data. Exploration of different characteristics, stable/unstable in-place/allocating and more. This analysis focuses on a lesser known and talked about aspect. How do implementations deal with a user-defined comparison function that implements arbitrary logic, may not implement a strict weak ordering, may leave the function without returning and can modify values as they are being compared.

The words sort implementation and sort algorithm, are expressly *not* used interchangeably. Practically all modern implementations are hybrids, using multiple sort algorithms.

## Safe-to-use abstractions

The Rust language has a concept of safe and unsafe interfaces. A safe interface is sound if the implementation avoids UB no matter how it is used, a property that composes. An implementation composed entirely of the usage of other safe interfaces is considered safe. However in the presence of other adjacent unsafe code, further considerations arise. This puts the burden on the interface designers and implementers, instead of the users. The C++ standard has a similar concept, called wide and narrow contracts, and while conceptually it's possible to design interfaces in C++ who's usage cannot lead to UB, common abstraction including standard library types like `std::vector` or algorithms like `std::sort` do not make such promises.

With the exception of rust_crumsort_rs_unstable, all Rust sort implementations considered in this analysis use unsafe abstractions in their implementations.

### Ord safety

What happens if the user-defined type or comparison function does not implement a strict weak ordering? The C++ standard library sort interface makes it trivial to trigger this case:

C++:

```cpp
sort(data.begin(), data.end(), [](const auto& a, const auto& b) {
    return a <= b; // correct would be a < b.
});
```

The Rust standard library sort interface avoids this problem in many cases, by requiring the user-defined comparison function to return the [`Ordering`](https://doc.rust-lang.org/std/cmp/enum.Ordering.html) type instead of a bool, but it is still possible:

Rust:

```rust
data.sort_by(|a, b| {
    if a == b {
        return Ordering::Less;
    }

    a.cmp(b)
});
```

The question what happens if the comparison function does not implement a strict weak ordering can be answered by constructing [experiments](https://github.com/Voultapher/sort-research-rs/blob/1b17ebcdaba9fe6988f09028d45da5b228a7e46e/sort_test_tools/src/tests.rs#L1025) and measuring the outcomes for various implementations. The question what *should* happen is trickier to answer. Adjacent is the question what is and isn't allowed to happen in order to avoid UB in every scenario.

Say the user wants to sort this input of integers:

```
[6, 3, 3, 2, 9, 1]
```

By mistake a comparison function is provided which does implement the required strict weak ordering. What are possible outcomes?

```
A: [2, 3, 9, 3, 1, 6]
B: [3, 2, 1, 3, 9, 6] + exception/panic
C: [1, 3, 3, 9, 9, 6]
D: [3, 3, 0, 0, 7, 9]
E: Runs forever
F: UB
```

By definition the result cannot be the input sorted by the predicate, the concept of "sorted" is nonsense without strict weak ordering. Yet not all possible outcomes are equal. Variant A returns after some time to the user and leaves the input in an unspecified order. The set of elements remains the same. Variant B is the same, with addition of raising
an exception in C++ and a panic in Rust informing the user of the logic bug in their program. Variant C also returns after some time but duplicated some elements and "looses" some elements. Variant D "invents" new elements that were never found in the original input. Variant E never returns to the user. And Variant F could be a wide range things like an out-of-bounds read that causes a CPU MMU exception, illegal CPU instructions, stack smashing,altering unrelated program state and more.

If the sort operation is understood as a series of swaps, C, D and E can all be quite surprising. How could they lead to UB?

- **C**: The duplication usually happens at the bit level, ignoring type semantics. If the element type is for example a `unique_ptr<int32_t>`/`Box<i32>`, these types assume unique ownership of an allocation. And their destructors will hand a pointer to the allocator for freeing. A bitwise copy results in use-after-free UB, most likely in the form of a double-free.
- **D**: Same as Variant C, with the addition of arbitrary UB usually caused by interpreting uninitialized memory as a valid occupancy of a type.
- **E**: Maybe UB, LLVM specifies such infinite loops without side-effect as UB, as does C++.

### Exception safety

C++ and Rust are both languages with scope based destructors (RAII), and stack unwinding. Together they prove a powerful abstraction for manual memory management. At the same time, they can make implementing generic code more complex. Every single point in the sort implementation that calls the user-provided comparison function, must assume that the call may return via an exception in C++ or panic in Rust.

C++:

```cpp
sort(data.begin(), data.end(), [](const auto& a, const auto& b) {
    if (some_condition(a, b)) {
        throw std::runtime_error{"unwind"};
    }

    return a < b;
});
```

Rust:

```rust
data.sort_by(|a, b| {
    if some_condition(a, b) {
        panic!("unwind");
    }

    a.cmp(b)
});
```

In practice a lack of exception safety manifests itself in the variants C and or D described in the section about Ord safety.

### Observation safety

Both C++ and Rust offer ways to mutate a value through a const/shared reference. In Rust this is called interior mutability. C++ achieves this with the help of the `mutable` type specifier, while Rust builds safe-to-use abstractions on top of the language builtin `UnsafeCell`. As a consequence of this it's possible to observe every call to the user-provided comparison function as a stack value modification. However, as soon as auxiliary memory, be it stack or heap, is used, unsafe bitwise duplications of the object are performed. If such a duplicated element is used as input to the user-provided comparison function, it may be modified in a way that must be observed when the sort completes, either by returning normally or by raising an exception/panic. A benign scenario with surprising consequences would be counting the comparisons performed by incrementing a counter held within the elements inside each call to the user-provided comparison function. If the property of guaranteed observable comparison is not met, the result may be wildly inaccurate in describing how many times the user-provided comparison function has been called. A more problematic scenario invoking UB would be, a user-defined type holding a pointer that is conditionally freed and set to null as part of the user-provided comparison function. If this modification is not observed after the sort completes, code that relies on a null-pointer check to see if it was already freed will run into use-after-free UB.

C++:

```cpp
struct ValWithPtr {
    int32_t val;
    mutable uint8_t* buffer;
    size_t buffer_len;

    ~ValWithPtr() {
        if (buffer) {
            free(buffer);
        }
    }
};

std::sort(data, data + len,  [&some_condition](const auto& a, const auto& b) {
    if (some_condition(a, b)) {
        free(a.buffer);
        a.buffer = nullptr;
        free(b.buffer);
        b.buffer = nullptr;
    }

    return a.val < b.val;    
});
```

Rust:

```rust
pub struct ValWithPtr {
    val: i32,
    buffer: Cell<Option<Box<[u8]>>>,
}

data.sort_by(|a, b| {
    if some_condition(a, b) {
        a.buffer.set(None);
        b.buffer.set(None);
    }

    a.val.cmp(&b.val)
});
```

The C language has no constructs that would allow safe modifications through a const/shared pointer, as such the tested C based sort implementations understandably fail this property. 


## Results

### Tested sort implementations

#### Stable

```
- rust_std_stable             | `slice::sort` https://github.com/rust-lang/rust (Vendored mid 2022)
- rust_wpwoodjr_stable        | https://github.com/wpwoodjr/rust-merge-sort (Vendored mid 2022)
- rust_glidesort_stable       | https://github.com/orlp/glidesort (version 0.1.2)
- cpp_std_gnu_stable          | libstdc++ `std::stable_sort` (gcc 12.2)
- cpp_std_libcxx_stable       | libc++ `std::stable_sort` (clang 15.0)
- cpp_std_msvc_stable         | MSVC (runtime library version 14.30)
- cpp_powersort_stable        | https://github.com/sebawild/powersort (Vendored mid 2022)
- cpp_powersort_4way_stable   | https://github.com/sebawild/powersort (Vendored mid 2022)
- c_fluxsort_stable           | https://github.com/scandum/fluxsort (Vendored early 2023)
```

#### Unstable

```
- rust_std_unstable           | `slice::sort_unstable` https://github.com/rust-lang/rust (Vendored mid 2022)
- rust_dmsort_unstable        | https://github.com/emilk/drop-merge-sort (version 1.0)
- rust_ipnsort_unstable       | https://github.com/Voultapher/sort-research-rs/tree/main/ipnsort (2fa4e4f)
- rust_crumsort_rs_unstable   | https://github.com/google/crumsort-rs (version 0.1)
- cpp_std_gnu_unstable        | libstdc++ `std::sort` (gcc 12.2)
- cpp_std_libcxx_unstable     | libc++ `std::sort` (clang 15.0)
- cpp_std_msvc_unstable       | MSVC (runtime library version 14.30)
- cpp_pdqsort_unstable        | https://github.com/orlp/pdqsort (Vendored mid 2022)
- cpp_ips4o_unstable          | https://github.com/ips4o/ips4o (Vendored mid 2022)
- cpp_blockquicksort_unstable | https://github.com/weissan/BlockQuicksort (Vendored mid 2022)
- c_crumsort_unstable         | https://github.com/scandum/crumsort (Vendored early 2023)
```

### Property analysis

Properties:

- **Functional**: Does the implementation successfully pass the test suite of different input patterns and supported types?
- **Generic**: Does the implementation support arbitrary user-defined types?
- **Ord safety**: What happens if the user-defined type or comparison function does not implement a strict weak ordering. E.g. in C++ your comparison function does `[](const auto& a, const auto& b) { return a.x <= b.x; }`? O == unspecified order but original elements, E == exception/panic and unspecified order but original elements, L == infinite loop, C == crash, e.g. heap-buffer-overflow (UB), D unspecified order with duplicates. Only O and E are safe.
- **Exception safety**: What happens, if the user provided comparison function throws an exception/panic? ✅ means it retains the original input set in an unspecified order, 🚫 means it may have duplicated elements in the input.
- **Observable comp**: If the type has interior mutability, will every modification caused by calling the user-defined comparison function with const/shared references be visible in the input, after the sort function returns 1: normally 2: panic. If exception safety is not given, it is practically impossible to achieve 2. here.
- **Miri**: Does the test-suite pass if run under [Miri](https://github.com/rust-lang/Miri)? S: using the Stacked Borrows aliasing model. T: using the Tree Borrows aliasing model.

| Name                         | Functional | Generic | Ord safety | Exception safety | Observation safety | Miri            |
|------------------------------|------------|---------|------------|------------------|--------------------|-----------------|
| rust_std_stable              | ✅         | ✅      | O ✅       | ✅               | 1: ✅ 2: ✅        | S: ✅ T: ✅     |
| rust_wpwoodjr_stable         | ✅         | ✅      | O ✅       | ✅               | 1: ✅ 2: ✅        | S: 🚫 T: ✅     |
| rust_glidesort_stable        | ✅         | ✅      | O ✅       | ✅               | 1: ✅ 2: ✅        | S: ✅ T: ✅     |
| cpp_std_gnu_stable           | ✅         | ✅      | C 🚫       | 🚫               | 1: ✅ 2: 🚫        | -               |
| cpp_std_libcxx_stable        | ✅         | ✅      | O ✅       | 🚫               | 1: ✅ 2: 🚫        | -               |
| cpp_std_msvc_stable          | ✅         | ✅      | C 🚫       | 🚫               | 1: ✅ 2: 🚫        | -               |
| cpp_powersort_stable         | ✅         | ⚠️ (1)  | O ✅       | 🚫               | 1: ✅ 2: 🚫        | -               |
| cpp_powersort_4way_stable    | ✅         | ⚠️ (2)  | O ✅       | 🚫               | 1: ✅ 2: 🚫        | -               |
| c_fluxsort_stable            | ✅         | ⚠️ (3)  | C 🚫       | 🚫 (5)           | 1: 🚫 2: 🚫 (7)    | -               |
| rust_std_unstable            | ✅         | ✅      | O ✅       | ✅               | 1: ✅ 2: ✅        | S: ✅ T: ✅     |
| rust_dmsort_unstable         | ✅         | ✅      | O ✅       | ✅               | 1: ✅ 2: 🚫        | S: 🚫 T: ⚠️ (8) |
| rust_ipnsort_unstable        | ✅         | ✅      | O or E ✅  | ✅               | 1: ✅ 2: ✅        | S: ✅ T: ✅     |
| rust_crumsort_rs_unstable    | ✅         | ⚠️ (4)  | D 🚫       | 🚫 (6)           | 1: -  2: -         | S: ⚠️ T: ⚠️ (8) |
| cpp_std_gnu_unstable         | ✅         | ✅      | C 🚫       | 🚫               | 1: ✅ 2: 🚫        | -               |
| cpp_std_libcxx_unstable      | ✅         | ✅      | L 🚫       | 🚫               | 1: ✅ 2: 🚫        | -               |
| cpp_std_msvc_unstable        | ✅         | ✅      | C 🚫       | 🚫               | 1: ✅ 2: 🚫        | -               |
| cpp_pdqsort_unstable         | ✅         | ✅      | L or C 🚫  | 🚫               | 1: ✅ 2: 🚫        | -               |
| cpp_ips4o_unstable           | ✅         | ✅      | C 🚫       | 🚫               | 1: 🚫 2: 🚫        | -               |
| cpp_blockquicksort_unstable  | ✅         | ✅      | C 🚫       | 🚫               | 1: ✅ 2: 🚫        | -               |
| c_crumsort_unstable          | ✅         | ⚠️ (3)  | C 🚫       | 🚫 (5)           | 1: 🚫 2: 🚫 (7)    | -               |

Footnotes:

1. cpp_powersort_stable uses `vector::resize` for it's buffer, requiring that `T` is default constructible.
2. cpp_powersort_4way_stable uses `vector::resize` for it's buffer, requiring that `T` is default constructible. cpp_powersort_4way_stable offers many configuration options, one of them is a template parameter called `mergingMethod`. `GENERAL_BY_STAGES` supports all user-defined types, but is relatively slow. `WILLEM_TUNED` is faster but requires a sentinel value for example `u64::MAX`, however this has the effect that the implementation can no longer correctly sort slices that contain the sentinel, making it unsuitable for a general purpose sort.
3. c_fluxsort_stable and c_crumsort_unstable use auxiliary stack and heap memory that may be under-aligned for types with alignment larger than fundamental alignment. The sort interface requires either, [large performance sacrifices](https://github.com/Voultapher/sort-research-rs/blob/main/writeup/intel_avx512/text.md#c-sort-interface) or source level modification.
4. rust_crumsort_rs_unstable limits itself to types that implement the `Copy` Trait, this includes types like integers but excludes all types with user-defined destructors like `String` and currently types with interior mutability like `Cell<i32>`.
5. c_fluxsort_stable and c_crumsort_unstable are developed as C based sorts. C has no concept of exceptions, or stack unwinding. So this property is only relevant if the code is compiled as C++ code.
6. By limiting itself to types with trivial destructors rust_crumsort_rs_unstable avoids UB as a direct consequence of a panic during a comparison. However is breaks the assumption that calling `sort` will retain the original set of elements after the call, which can lead to UB in adjacent logic that uses unsafe code otherwise considered sound. Even though rust_crumsort_rs_unstable uses zero lines of unsafe Rust, failing to uphold an intuitive and currently under documented property of sort it can break other unsafe code that relies on this assumption.
7. c_fluxsort_stable and c_crumsort_unstable are developed as C based sorts. C has no concept of interior mutability.
8. Passes all tests except those that failed for reasons not unique to the kind of checks Miri performs.

### Observations

- Some notionally generic implementations, do not support every kind of user-defined type possible in their respective implementation language. This includes cpp_powersort_stable, cpp_powersort_4way_stable, c_fluxsort_stable, rust_crumsort_rs_unstable, c_crumsort_unstable.
- With the exception of rust_crumsort_rs_unstable ([reported issue](https://github.com/google/crumsort-rs/issues/2)), which is a port of the C based c_crumsort_unstable, all Rust based implementations are Ord safe. The only C and C++ implementations that avoid UB in such scenarios are, cpp_std_libcxx_stable and the powersort family of implementations. It's unclear if this is by design or by accident. rust_ipnsort_unstable goes further in terms of usability by raising a recoverable and deterministic panic if an Ord violation is detected, informing the user of a logic bug.
- With the exception of rust_crumsort_rs_unstable ([reported issue](https://github.com/google/crumsort-rs/issues/1)), which is a port of the C based c_crumsort_unstable, all Rust based implementations are Exception safe. The fact that none of the tested C++ implementations are exception safe, may seem to indicate that such a property is impossible in C++. However that is easily refutable by showing that this issue only arises with the usage of auxiliary memory. And the usage of auxiliary memory can be reduced in the temporal domain to the temporary value used for swapping 2 elements. Exception safe sort implementation can exist in C++, even with no runtime overhead for unaffected types by using scope guards. The lack of exception safety is indicative of different interface and usage expectations by the library authors. And while it may seem tempting to switch to some faster implementation for types that are `std::is_trivially_destructible`/`!core::mem::needs_drop`, doing so would still risk problems for types such as raw pointers, [which are trivially destructible](https://godbolt.org/z/rWe9rn9G3). And may be used to sort by indirection. A significant amount of real world code assumes that a sort involves changing the order of elements. Significantly less code handles the reality that widely used implementations may also change the set of provided elements, replacing some with duplicates of others. Which in the case of pointers quickly leads to UB such as double-free use-after-free. Even special casing for builtin integer types could lead to problems, in the presence of a user-defined comparison function. These integers may be interpreted as pointers, even though such code has [pointer-provenance issues](https://faultlore.com/blah/fix-rust-pointers/), such code is currently a common occurrence in real world code.
- With the exception of cpp_ips4o_unstable, all Rust and C++ implementations provide the first kind of observation safety. C implementations only need to care about it, if they are compiled as C++ code. The presence of observation safety in all C++ std library implementations indicates, that such usage is found in real world code the vendors care about, even if the standard leaves this behavior [unspecified](https://eel.is/c++draft/sort).
- rust_dmsort_unstable shows that even if exception safety and the first kind of observation safety are given, having the second kind of observation safety is not a guarantee. ([reported issue](https://github.com/emilk/drop-merge-sort/issues/23))
- Similar to C++, the language rules of Rust are defined in terms of an abstract machine and not one concrete implementation. And UB is a violation of said rules, a violation that is assumed will never happen. Thus it is possible to perform certain optimizations under the assumptions that certain properties that are tricky or impossible to prove for the compiler are true. For more information about this, see [this talk](https://youtu.be/yG1OZ69H_-o) by Chandler Carruth. Miri is a tool to run Rust code inside a virtual machine that tries to find UB in unsafe code, by strictly modeling said abstract machine, checking as many properties as possible. It checks alignment, aliasing and more. Aliasing in particular is still an open topic in Rust. Miri uses by default the [Stacked Borrows](https://plv.mpi-sws.org/rustbelt/stacked-borrows/paper.pdf) aliasing model for unsafe Rust code. And while code may fail under Miri it is not always clear if that is a serious concern. This can only be answered once Rust has settled on an aliasing model. With the introduction of [Tree Borrows](https://perso.crans.org/vanille/treebor/) there are now two different aliasing models that can be used with Miri. Inadvertently these results validate one of the goals of Tree Borrows, having an aliasing model that avoids surprising authors of unsafe code. Both rust_wpwoodjr_stable and rust_dmsort_unstable fail the test suite with Stacked Borrows and pass it under the rules set by Tree Borrows.

## Author's conclusion and opinion

As demonstrated [here](https://github.com/Voultapher/sort-research-rs/blob/main/writeup/intel_avx512/text.md) and [here](https://github.com/Voultapher/sort-research-rs/blob/main/writeup/glidesort_perf_analysis/text.md), the current Rust standard library implementations outperform their C++ counterparts. And that despite providing significantly safer to use implementations. glidesort and ipnsort demonstrate that these properties can be upheld even in state-of-the-art high-performance implementations. The sort implementations in the C++ standard libraries are usually quite old, which can explain their poor performance. Yet even relatively new C++ implementations such as ips4o disregard usage safety completely, even regressing Observation safety compared to the tested standard library implementations. The new and so far untested [libc++ implementation](https://danlark.org/2022/04/20/changing-stdsort-at-googles-scale-and-beyond) shows awareness of some of the analyzed safety properties, mainly Ord safety, but fails to find a way to guarantee UB free usage. Only going as far as performing a sampled strict weak ordering violation check for Debug builds. Which comes as bit of a surprise given the release date of 2022, five years after the pdqsort-derived unstable sort in Rust was [merged in 2017](https://github.com/rust-lang/rust/pull/40601). I see no reason why a straight port from Rust to C++ wouldn't have been possible while satisfying their requirements. The author Danila Kutenin even mentions the Rust implementation, so I assume they are aware of it. To me the Results across all tested implementations is indicative of a pervasive mindset in the C++ world, that argues it's the users responsibility to be careful, even if that has been [proven impossible](https://alexgaynor.net/2020/may/27/science-on-memory-unsafety-and-security/) at scale. Personally I've spent several days debugging some code at work that broke in very strange ways, and was caused by accidentally writing `<=` instead of `<` in a comparison function, affecting logic in a completely different place. Often safety and performance are characterized as a set of zero sum tradeoffs, yet often it's possible to find better tradeoffs who's holistic properties improve upon a previously seen "either or". Taking the 1 to N relationship of foundational library authors to library users into consideration, the impact of safe-to-use abstractions should become apparent.

## Thanks

Thank you Klaus Iglberger for providing detailed feedback and valuable suggestions on improving the readability of this writeup.
