# Safety vs Performance. A case study of C, C++ and Rust sort implementations.

Author: Lukas Bergdoll @Voultapher  
Date: 05-10-2023 (DD-MM-YYYY)

This is an analysis of sort implementations and their ability, or lack thereof, to avoid undefined behavior (UB) under various usage scenarios, and whether this affects their performance.

TL;DR: The combination of complex generic implementations striving for performance with arbitrary logic in user-defined comparison functions, makes generic high-performance sort implementations particularly difficult to implement in a way that avoids UB under every usage scenario. Even a sort implementation using only memory-safe abstractions is no guarantee of UB free adjacent logic. Overall no correlation between performance and safety could be found, nor whether safe or unsafe internal abstractions are used. However a strong correlation between being implemented for C or C++ users and a lack of safety presents itself.

---

Bias disclaimer. The author of this analysis is the author of ipnsort.


## Introduction

Implementing a sort operation with the help of computers goes back to the early 1950s. The problem statement is deceptively simple. Take a list of elements and use a comparison function that implements a [strict weak ordering](https://en.wikipedia.org/wiki/Weak_ordering#Strict_weak_orderings) to swap elements until it's sorted. Now, 70 years later new and more resource-efficient ways to implement this operation are still being discovered. It's an active field of study in science, [BlockQuicksort](https://arxiv.org/pdf/1604.06697.pdf) 2016, [ips4o](https://arxiv.org/pdf/1705.02257.pdf) 2017, [pdqsort](https://arxiv.org/pdf/2106.05123.pdf) 2021, [Multiway Powersort](https://arxiv.org/pdf/2209.06909.pdf) 2022, and many more. There are various directions science is exploring. Efficient sort implementations running single threaded on modern superscalar, out-of-order and speculative CPUs. Efficient implementations running on multiple such threads. Implementations running on massively parallel in-order GPUs. Exploration of better best-case, average-case and worst-case runtime. Exploiting existing patterns in the input data. Exploration of different characteristics, stable/unstable in-place/allocating and more. This analysis focuses on a lesser known and talked about aspect. How do implementations deal with a user-defined comparison function that implements arbitrary logic, may not implement a strict weak ordering, may leave the function without returning and can modify values as they are being compared.

The words sort implementation and sort algorithm, are explicitly *not* used interchangeably. Practically all modern implementations are hybrids, using multiple sort algorithms.

## Safe-to-use abstractions

The Rust language has a concept of safe and unsafe interfaces. A safe interface is sound if the implementation avoids UB no matter how it is used, a property that composes. An implementation composed entirely of the usage of other safe interfaces is considered safe. However, in the presence of other adjacent unsafe code, further considerations arise. This puts the burden on the interface designers and implementers, instead of the users. The C++ standard has a similar concept, called wide and narrow contracts, and while conceptually it's possible to design interfaces in C++ whose usage cannot lead to UB, common abstraction including standard library types like `std::vector` or algorithms like `std::sort` do not make such promises.

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

By mistake a comparison function is provided which does not implement the required strict weak ordering. What are possible outcomes?

```
A: [2, 3, 9, 3, 1, 6]
B: [3, 2, 1, 3, 9, 6] + exception/panic
C: [1, 3, 3, 9, 9, 6]
D: [3, 3, 0, 0, 7, 9]
E: Runs forever
F: UB
```

By definition the result cannot be the input sorted by the predicate, the concept of "sorted" is nonsense without strict weak ordering. Yet not all possible outcomes are equal. Variant A returns after some time to the user and leaves the input in an unspecified order. The set of elements remains the same. Variant B is the same, with addition of raising
an exception in C++ and a panic in Rust informing the user of the logic bug in their program. Variant C also returns after some time but duplicated some elements and "looses" some elements. Variant D "invents" new elements that were never found in the original input. Variant E never returns to the user. And Variant F could be a wide range things like an out-of-bounds read that causes a CPU MMU exception, illegal CPU instructions, stack smashing, altering unrelated program state and more.

If the sort operation is understood as a series of swaps, C, D, E and F can all be quite surprising. How could they lead to UB?

- **C**: The duplication usually happens at the bit level, ignoring type semantics. If the element type is for example a `unique_ptr<int32_t>`/`Box<i32>`, these types assume unique ownership of an allocation. And their destructors will hand a pointer to the allocator for freeing. A bitwise copy results in use-after-free UB, most likely in the form of a double-free.
- **D**: Same as Variant C, with the addition of arbitrary UB usually caused by interpreting uninitialized memory as a valid occupancy of a type.
- **E**: Maybe UB, LLVM specifies such infinite loops without side-effect as UB, as does C++.

### Exception safety

Exception safety encompasses the various guarantees of correctness that can be provided in the presence of exception. In the concrete case of sort implementations that accept user-provided comparison functions, exception safety expresses the possible behavior when the comparison function can throw an exception.

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

The weakest guarantee is that an exception does not directly lead to undefined behavior. The strongest guarantee is "transactional exception safety", that guarantees that in the case an operation fails, then the state of the program is reverted to the one before the failed operation was attempted. This strongest guarantee is typically both too strong and too performance intensive, so very rarely observed in practice.

For the purpose of this benchmark, the guarantee we are interested in will be denoted "intuitive exception safety". It encompasses the natural behavior a user that is not especially aware of the internal of sort algorithms may expect when a sort routine is interrupted. As the observable side-effect of a sort routine is to reorder elements of the input, intuitive exception safety expresses the guarantee that in the face of an exception, elements from the input may only be partially reordered, as if the sort process had been interrupted. In particular, this excludes modifying elements of the input, for instance duplicating some of them, removing some of them, or, in the case of C++, leaving some elements in the "moved out" state.

We choose intuitive exception safety as the desired property for sort algorithms, because failure to uphold this property can indirectly cause UB in user code by violating invariants enforced in the rest of the code.

As examples, consider the following two situations:

1. The user is sorting a vector of move-only types like `std::unique_ptr`. If the user's code guarantees that, by construction, the pointers in that vector are not null, then it is permissible in user code to rely on this invariant and omit null checks. However, this is fraught in the presence of a sort implementation that doesn't verify the intuitive exception safety property: the input may unexpectedly contain null pointers after the comparison function throws.
2. The user is sorting a vector of natural numbers that serve as indices in a graph-like structures. If the user's code guarantees that, by construction, the indices in that vector are never repeated, then it is permissible in user code torely on this invariant and delete the node associated with each index without checking if it was previously deleted. However, this is fraught in the presence of a sort implementation that doesn't verify the intuitive exception safety property: for trivially-copyable types, the input may unexpectedly contain duplicated indices after the comparison function throws.

These issues are not theoretical: even assuming a world filled exclusively with C++ types following best practices, where duplicating integers will not directly lead to UB, it can still easily break adjacent assumptions made about a sort operation only re-arranging elements and not duplicating them, as shown [here](https://github.com/google/crumsort-rs/issues/1).

### Observation safety

Both C++ and Rust offer ways to mutate a value through a const/shared reference. In Rust this is called interior mutability. C++ achieves this with the help of the `mutable` type specifier, while Rust builds safe-to-use abstractions on top of the language builtin `UnsafeCell`. As a consequence of this it's possible to observe every call to the user-provided comparison function as a stack value modification. As soon as auxiliary memory, be it stack or heap, is used, unsafe bitwise duplications of the object are performed. If such a duplicated element is used as input to the user-provided comparison function, it may be modified in a way that must be observed when the sort completes, either by returning normally or by raising an exception/panic. A benign scenario with surprising consequences would be counting the comparisons performed by incrementing a counter held within the elements inside each call to the user-provided comparison function. If the property of guaranteed observable comparison is not met, the result may be wildly inaccurate in describing how many times the user-provided comparison function has been called. A more problematic scenario invoking UB would be, a user-defined type holding a pointer that is conditionally freed and set to null as part of the user-provided comparison function. If this modification is not observed after the sort completes, code that relies on a null-pointer check to see if it was already freed will run into use-after-free UB.

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
- cpp_std_gnu_stable          | libstdc++ `std::stable_sort` (gcc 13.2.1)
- cpp_std_libcxx_stable       | libc++ `std::stable_sort` (clang 16.0.6)
- cpp_std_msvc_stable         | MSVC `std::stable_sort` (runtime library version 14.30)
- cpp_powersort_stable        | https://github.com/sebawild/powersort (Vendored mid 2022)
- cpp_powersort_4way_stable   | https://github.com/sebawild/powersort (Vendored mid 2022)
- c_fluxsort_stable           | https://github.com/scandum/fluxsort (Vendored early 2023)
```

#### Unstable

```
- rust_std_unstable           | `slice::sort_unstable` https://github.com/rust-lang/rust (Vendored mid 2022)
- rust_dmsort_unstable        | https://github.com/emilk/drop-merge-sort (version 1.0)
- rust_ipnsort_unstable       | https://github.com/Voultapher/sort-research-rs/tree/main/ipnsort (757478b)
- rust_crumsort_rs_unstable   | https://github.com/google/crumsort-rs (version 0.1)
- cpp_std_gnu_unstable        | libstdc++ `std::sort` (gcc 13.2.1)
- cpp_std_libcxx_unstable     | libc++ `std::sort` (clang 16.0.6)
- cpp_std_msvc_unstable       | MSVC `std::sort` (runtime library version 14.30)
- cpp_pdqsort_unstable        | https://github.com/orlp/pdqsort (Vendored mid 2022)
- cpp_ips4o_unstable          | https://github.com/ips4o/ips4o (Vendored mid 2022)
- cpp_blockquicksort_unstable | https://github.com/weissan/BlockQuicksort (Vendored mid 2022)
- c_crumsort_unstable         | https://github.com/scandum/crumsort (Vendored early 2023)
```

### Property analysis

Properties:

- **Functional**: Does the implementation successfully pass the test suite of different input patterns and supported types?
- **Generic**: Does the implementation support arbitrary user-defined types?
- **Ord safety**: What happens if the user-defined type or comparison function does not implement a strict weak ordering. E.g. in C++ your comparison function does `[](const auto& a, const auto& b) { return a.x <= b.x; }`? O == unspecified order but original elements, E == exception/panic and unspecified order but original elements, U == Undefined Behavior usually out-of-bounds read and write, D unspecified order with duplicates. Only O and E are safe.
- **Exception safety**: What happens, if the user provided comparison function throws an exception/panic? ✅ means it retains the original input set in an unspecified order, upholding the intuitiv exception safety property outlined above 🚫 means it may have duplicated or moved-out elements in the input.
- **Observable comp**: If the type has interior mutability, will every modification caused by calling the user-defined comparison function with const/shared references be visible in the input, after the sort function returns 1: normally 2: panic. If exception safety is not given, it is practically impossible to achieve 2. here.
- **Miri**: Does the test-suite pass if run under [Miri](https://github.com/rust-lang/Miri)? S: using the Stacked Borrows aliasing model. T: using the Tree Borrows aliasing model.

| Name                         | Functional | Generic | Ord safety | Exception safety | Observation safety | Miri            |
|------------------------------|------------|---------|------------|------------------|--------------------|-----------------|
| rust_std_stable              | ✅         | ✅      | O ✅       | ✅               | 1: ✅ 2: ✅        | S: ✅ T: ✅     |
| rust_wpwoodjr_stable         | ✅         | ✅      | O ✅       | ✅               | 1: ✅ 2: ✅        | S: 🚫 T: ✅     |
| rust_glidesort_stable        | ✅         | ✅      | O ✅       | ✅               | 1: ✅ 2: ✅        | S: ✅ T: ✅     |
| cpp_std_gnu_stable           | ✅         | ✅      | U 🚫       | 🚫               | 1: ✅ 2: 🚫        | -               |
| cpp_std_libcxx_stable        | ✅         | ✅      | O ✅       | 🚫               | 1: ✅ 2: 🚫        | -               |
| cpp_std_msvc_stable          | ✅         | ✅      | U 🚫       | 🚫               | 1: ✅ 2: 🚫        | -               |
| cpp_powersort_stable         | ✅         | ⚠️ (1)  | O ✅       | 🚫               | 1: ✅ 2: 🚫        | -               |
| cpp_powersort_4way_stable    | ✅         | ⚠️ (2)  | O ✅       | 🚫               | 1: ✅ 2: 🚫        | -               |
| c_fluxsort_stable            | ✅         | ⚠️ (3)  | U 🚫       | 🚫 (6)           | 1: 🚫 2: 🚫 (8)    | -               |
| rust_std_unstable            | ✅         | ✅      | O ✅       | ✅               | 1: ✅ 2: ✅        | S: ✅ T: ✅     |
| rust_dmsort_unstable         | ✅         | ✅      | O ✅       | ✅               | 1: ✅ 2: 🚫        | S: 🚫 T: ⚠️ (9) |
| rust_ipnsort_unstable        | ✅         | ✅      | O or E ✅  | ✅               | 1: ✅ 2: ✅        | S: ✅ T: ✅     |
| rust_crumsort_rs_unstable    | ✅         | ⚠️ (4)  | D 🚫       | 🚫 (7)           | 1: -  2: -         | S: ⚠️ T: ⚠️ (9) |
| cpp_std_gnu_unstable         | ✅         | ✅      | U 🚫       | 🚫               | 1: ✅ 2: 🚫        | -               |
| cpp_std_libcxx_unstable      | ✅         | ✅      | U 🚫       | 🚫               | 1: ✅ 2: 🚫        | -               |
| cpp_std_msvc_unstable        | ✅         | ✅      | U 🚫       | 🚫               | 1: ✅ 2: 🚫        | -               |
| cpp_pdqsort_unstable         | ✅         | ✅      | U 🚫       | 🚫               | 1: ✅ 2: 🚫        | -               |
| cpp_ips4o_unstable           | ✅         | ⚠️ (5)  | U 🚫       | 🚫               | 1: 🚫 2: 🚫        | -               |
| cpp_blockquicksort_unstable  | ✅         | ⚠️ (5)  | U 🚫       | 🚫               | 1: ✅ 2: 🚫        | -               |
| c_crumsort_unstable          | ✅         | ⚠️ (3)  | U 🚫       | 🚫 (6)           | 1: 🚫 2: 🚫 (8)    | -               |

Footnotes:

1. cpp_powersort_stable uses `vector::resize` for it's buffer, requiring that `T` is default constructible.
2. cpp_powersort_4way_stable uses `vector::resize` for it's buffer, requiring that `T` is default constructible. cpp_powersort_4way_stable offers many configuration options, one of them is a template parameter called `mergingMethod`. `GENERAL_BY_STAGES` supports all user-defined types, but is relatively slow. `WILLEM_TUNED` is faster but requires a sentinel value for example `u64::MAX`, however this has the effect that the implementation can no longer correctly sort slices that contain the sentinel, making it unsuitable for a general purpose sort.
3. c_fluxsort_stable and c_crumsort_unstable use auxiliary stack and heap memory that may be under-aligned for types with alignment larger than fundamental alignment. The sort interface requires either, [large performance sacrifices](https://github.com/Voultapher/sort-research-rs/blob/main/writeup/intel_avx512/text.md#c-sort-interface) or source level modification.
4. rust_crumsort_rs_unstable limits itself to types that implement the `Copy` Trait, this includes types like integers but excludes all types with user-defined destructors like `String` and currently types with interior mutability like `Cell<i32>`.
5. cpp_ips4o_unstable and cpp_blockquicksort_unstable are implemented in a way that requires that `T` implements a by ref copy constructor. This is stricter than the C++ standard library type requirements.
6. c_fluxsort_stable and c_crumsort_unstable are developed as C based sorts. C has no concept of exceptions, or stack unwinding. So this property is only relevant if the code is compiled as C++ code.
7. By limiting itself to types with trivial destructors rust_crumsort_rs_unstable avoids UB as a direct consequence of a panic during a comparison. However is breaks the assumption that calling `sort` will retain the original set of elements after the call, which can lead to UB in adjacent logic that uses unsafe code otherwise considered sound. Even though rust_crumsort_rs_unstable uses zero lines of unsafe Rust, failing to uphold an intuitive and currently under documented property of sort it can break other unsafe code that relies on this assumption.
8. c_fluxsort_stable and c_crumsort_unstable are developed as C based sorts. C has no concept of interior mutability.
9. Passes all tests except those that failed for reasons not unique to the kind of checks Miri performs.

### Observations

- Some notionally generic implementations, do not support every kind of user-defined type possible in their respective implementation language. This includes cpp_powersort_stable, cpp_powersort_4way_stable, c_fluxsort_stable, rust_crumsort_rs_unstable, c_crumsort_unstable.
- With the exception of rust_crumsort_rs_unstable ([reported issue](https://github.com/google/crumsort-rs/issues/2)), which is a port of the C based c_crumsort_unstable, all Rust based implementations are Ord safe. The only C and C++ implementations that avoid UB in such scenarios are, cpp_std_libcxx_stable and the powersort family of implementations. It's unclear if this is by design or by accident. rust_ipnsort_unstable goes further in terms of usability by raising a recoverable and deterministic panic if an Ord violation is detected, informing the user of a logic bug.
- With the exception of rust_crumsort_rs_unstable ([reported issue](https://github.com/google/crumsort-rs/issues/1)), which is a port of the C based c_crumsort_unstable, all Rust based implementations are Exception safe. The fact that none of the tested C++ implementations are exception safe, may seem to indicate that such a property is impossible in C++. However that is easily refutable by showing that this issue only arises with the usage of auxiliary memory. And the usage of auxiliary memory can be reduced in the temporal domain to the temporary value used for swapping 2 elements. Exception safe sort implementation can exist in C++, even with no runtime overhead for unaffected types by using scope guards. The lack of exception safety is indicative of different interface and usage expectations by the library authors. And while it may seem tempting to switch to some faster implementation for types that are `std::is_trivially_destructible`/`!core::mem::needs_drop`, doing so would still risk problems for types such as raw pointers, [which are trivially destructible](https://godbolt.org/z/rWe9rn9G3). And may be used to sort by indirection. A significant amount of real world code assumes that a sort involves changing the order of elements. Significantly less code handles the reality that widely used implementations may also change the set of provided elements, replacing some with duplicates of others. Which in the case of pointers quickly leads to UB such as double-free use-after-free. Even special casing for builtin integer types could lead to problems, in the presence of a user-defined comparison function. These integers may be interpreted as pointers, even though such code has [pointer-provenance issues](https://faultlore.com/blah/fix-rust-pointers/), such code is currently a common occurrence in real world code.
- With the exception of cpp_ips4o_unstable, all Rust and C++ implementations provide the first kind of observation safety. C implementations only need to care about it, if they are compiled as C++ code. The presence of observation safety in all C++ std library implementations indicates, that such usage is found in real world code the vendors care about, even if the standard leaves this behavior [unspecified](https://eel.is/c++draft/sort).
- rust_dmsort_unstable shows that even if exception safety and the first kind of observation safety are given, having the second kind of observation safety is not a guarantee. ([reported issue](https://github.com/emilk/drop-merge-sort/issues/23))
- Similar to C++, the language rules of Rust are defined in terms of an abstract machine and not one concrete implementation. And UB is a violation of said rules, a violation that is assumed will never happen. Thus it is possible to perform certain optimizations under the assumptions that certain properties that are tricky or impossible to prove for the compiler are true. For more information about this, see [this talk](https://youtu.be/yG1OZ69H_-o) by Chandler Carruth. Miri is a tool to run Rust code inside a virtual machine that tries to find UB in unsafe code, by strictly modeling said abstract machine, checking as many properties as possible. It checks alignment, aliasing and more. Aliasing in particular is still an open topic in Rust. Miri uses by default the [Stacked Borrows](https://plv.mpi-sws.org/rustbelt/stacked-borrows/paper.pdf) aliasing model for unsafe Rust code. And while code may fail under Miri it is not always clear if that is a serious concern. This can only be answered once Rust has settled on an aliasing model. With the introduction of [Tree Borrows](https://perso.crans.org/vanille/treebor/) there are now two different aliasing models that can be used with Miri. Inadvertently these results validate one of the goals of Tree Borrows, having an aliasing model that avoids surprising authors of unsafe code. Both rust_wpwoodjr_stable and rust_dmsort_unstable fail the test suite with Stacked Borrows and pass it under the rules set by Tree Borrows.

## Performance

### Benchmark setup

```
Linux 6.5
rustc 1.75.0-nightly (187b8131d 2023-10-03)
clang version 16.0.6
gcc version 13.2.1
AMD Ryzen 9 5900X 12-Core Processor (Zen 3 micro-architecture)
CPU boost enabled.
```

Rust code built with `--release` and `lto=thin` and native code build settings can be found [here](https://github.com/Voultapher/sort-research-rs/blob/190b3ec27f616c139370285b2b8534d1c5eaec1b/build.rs).

Some sort implementations are adaptive, they will try to exploit existing patterns in the data to do less work. A breakdown of the benchmark patterns:

- `ascending`, numbers `0..len`
- `descending`, numbers `0..len` reversed
- `random`, random numbers generated by rand `StdRng::gen` [[2](https://github.com/rust-random/rand)]
- `random_d20`, uniform random numbers in the range `0..=20`
- `random_p5`, 95% 0 and 5% random data, not uniform
- `random_s95`, 95% sorted followed by 5% unsorted, simulates append -> sort
- `random_z1`, Zipfian distribution with characterizing exponent s == 1.0 [[3](https://en.wikipedia.org/wiki/Zipf%27s_law)]

### Results

Only 10k `u64` are tested on Zen 3. A more in depth look at performance of unstable sort implementations [here](https://github.com/Voultapher/sort-research-rs/blob/main/writeup/intel_avx512/text.md) and performance of stable sort implementations [here](https://github.com/Voultapher/sort-research-rs/blob/main/writeup/glidesort_perf_analysis/text.md). Beware updates to some of the tested implementations and toolchains make the results not directly comparable with the results below.

#### Stable

<img src="assets/stable-hot-u64-10k.png" width=810 />

#### Unstable

<img src="assets/unstable-hot-u64-10k.png" width=810 />


## libc++'s (libcxx) new `std::sort`

This is an attempt at untangling the update to libc++'s `std::sort`. A rough timeline to the best of the author's knowledge:

- March 2022 [PR for libc++ opened](https://reviews.llvm.org/D122780)
- April 2022 [blog post by Danila Kutenin](https://danlark.org/2022/04/20/changing-stdsort-at-googles-scale-and-beyond)
- December 2022 PR merged, on-track for LLVM 16 release
- April 2023 [change is reverted on feature branch due to Ord safety concerns](https://github.com/llvm/llvm-project/commit/9ec8096d0d50a353a5bc5a91064c6332bd634021)
- May 2023 [out-of-bounds read mitigations are added](https://github.com/llvm/llvm-project/commit/36d8b449cfc9850513bb2ed6c07b5b8cc9f1ae3a)

As it stands the updated `std::sort` is on track for LLVM 17. To add to the confusion, the PR title and blog post talk about a partition implementation based on [BlockQuickSort](https://github.com/weissan/BlockQuicksort), but the merged version uses a similar but notably different implementation derived from [bitsetsort](https://github.com/minjaehwang/bitsetsort).

## Author's conclusion and opinion

As seen in the benchmarks, the current Rust standard library unstable sort implementation outperforms the C++ standard library counterparts. And that despite providing a significantly safer to use implementation. glidesort and ipnsort demonstrate that these properties can be upheld even in state-of-the-art high-performance implementations. The sort implementations in the C++ standard libraries are usually quite old, which can explain their poor performance. Yet even relatively new C++ implementations such as ips4o disregard usage safety completely, even regressing Observation safety compared to the tested standard library implementations. The new and so far untested libc++ implementation shows awareness of some of the analyzed safety properties, mainly Ord safety, but fails to find a way to guarantee UB free usage. Only going as far as performing an optional opt-in out-of-bounds (OOB) check. Disregarding the issues of duplicate elements and exception safety. Which comes as bit of a surprise given the release date of 2022, five years after the pdqsort-derived unstable sort in Rust was [merged in 2017](https://github.com/rust-lang/rust/pull/40601). I see no reason why a straight port from Rust to C++ wouldn't have been possible while satisfying their requirements. The author Danila Kutenin even mentions the Rust implementation in their blog post, so I assume they are aware of it. To me the Results across all tested implementations is indicative of a pervasive mindset in the C and C++ world, that argues it's the users responsibility to be careful, even if that has been [proven impossible](https://alexgaynor.net/2020/may/27/science-on-memory-unsafety-and-security/) at scale. Personally I've spent several days debugging some code at work that broke in very strange ways, and was caused by accidentally writing `<=` instead of `<` in a comparison function, affecting logic in a completely different place. Often safety and performance are characterized as a set of zero sum tradeoffs, yet often it's possible to find better tradeoffs who's holistic properties improve upon a previously seen "either or". Taking the 1 to N relationship of foundational library authors to library users into consideration, the impact of safe-to-use abstractions should become apparent.

## Thanks

Thank you Klaus Iglberger for providing detailed feedback and valuable suggestions on improving the readability of this writeup.
