This PR is a followup to https://github.com/rust-lang/rust/pull/124032. It replaces the tests that test the various sort functions in the standard library with a test-suite developed as part of https://github.com/Voultapher/sort-research-rs. The current tests suffer a couple of problems:

- They don't cover important real world patterns that the implementations take advantage of and execute special code for.
- The input lengths tested miss out on code paths. For example, important safety property tests never reach the quicksort part of the implementation.
- The miri side is often limited to `len <= 20` which means it very thoroughly tests the insertion sort, which accounts for 19 out of 1.5k LoC.
- They are split into to core and alloc, causing code duplication and uneven coverage.
- The randomness is tied to a caller location, wasting the space exploration capabilities of randomized testing.

Most of these issues existed before https://github.com/rust-lang/rust/pull/124032, but they are intensified by it. One thing that is new and requires additional testing, is that the new sort implementations specialize based on type properties. For example `Freeze` and non `Freeze` execute different code paths.

Effectively there are three dimensions that matter:

- Input type
- Input length
- Input pattern

The ported test-suite tests various properties along all three dimensions, greatly improving test coverage. It side-steps the miri issue by preferring sampled approaches. For example the test that checks if after a panic the set of elements is still the original one, doesn't do so for every single possible panic opportunity but rather it picks one at random, and performs this test across a range of input length, which varies the panic point across them. This allows regular execution to easily test inputs of length 10k, and miri execution up to 100 which covers significantly more code. The randomness used is tied to a fixed - but random per process execution - seed. This allows for fully repeatable tests and fuzzer like exploration across multiple runs.

Structure wise, the tests are previously found in the core integration tests for `sort_unstable` and alloc unit tests for `sort`. The new test-suite was developed to be a purely black-box approach, which makes integration testing the better place, because it can't accidentally rely on internal access. Because unwinding support is required the tests can't be in core, even if the implementation is, so they are now part of the alloc integration tests. Are there architectures that can only build and test core and not alloc? If so, do such platforms require sort testing? For what it's worth the current implementation state passes miri `--target mips64-unknown-linux-gnuabi64` which is big endian.

The test-suite also contains tests for properties that were and are given by the current and previous implementations, and likely relied upon by users but weren't tested. For example `self_cmp` tests that the two parameters `a` and `b` passed into the comparison function are never references to the same object, which if the user is sorting for example a `&mut [Mutex<i32>]` could lead to a deadlock.

Instead of using the hashed caller location as rand seed, it uses seconds since unix epoch / 10, which given timestamps in the CI should be reasonably easy to reproduce, but also allows fuzzer like space exploration.

---

Test run-time changes:

Setup:

```
Linux 6.10
rustc 1.82.0-nightly (2c93fabd9 2024-08-15)
AMD Ryzen 9 5900X 12-Core Processor (Zen 3 micro-architecture)
CPU boost enabled.
```

Before core integration tests:

```
$ LD_LIBRARY_PATH=build/x86_64-unknown-linux-gnu/stage0-std/x86_64-unknown-linux-gnu/release/deps/ hyperfine build/x86_64-unknown-linux-gnu/stage0-std/x86_64-unknown-linux-gnu/release/deps/coretests-219cbd0308a49e2f
  Time (mean ± σ):     878.0 ms ±  27.5 ms    [User: 1312.4 ms, System: 65.8 ms]
  Range (min … max):   847.5 ms … 932.7 ms    10 runs
```

miri: `finished in 540.32s` using MIRIFLAGS="-Zmiri-disable-isolation" to get real time

After core integration tests:

TODO

Before alloc unit tests:

```
LD_LIBRARY_PATH=build/x86_64-unknown-linux-gnu/stage0-std/x86_64-unknown-linux-gnu/release/deps/ hyperfine build/x86_64-unknown-linux-gnu/stage0-std/x86_64-unknown-linux-gnu/release/deps/alloc-fd3dbe01b07af0bd
  Time (mean ± σ):     290.0 ms ±   4.4 ms    [User: 675.1 ms, System: 38.5 ms]
  Range (min … max):   285.3 ms … 297.2 ms    10 runs
```

miri: `finished in 261.42s` using MIRIFLAGS="-Zmiri-disable-isolation" to get real time

After alloc unit tests:

TODO

Before alloc integration tests:

```
$ LD_LIBRARY_PATH=build/x86_64-unknown-linux-gnu/stage0-std/x86_64-unknown-linux-gnu/release/deps/ hyperfine build/x86_64-unknown-linux-gnu/stage0-std/x86_64-unknown-linux-gnu/release/deps/alloctests-ed4a7ec5142cec3b
  Time (mean ± σ):     104.3 ms ±   3.1 ms    [User: 130.0 ms, System: 46.1 ms]
  Range (min … max):   100.8 ms … 117.5 ms    28 runs
```

miri: `finished in 187.72s` using MIRIFLAGS="-Zmiri-disable-isolation" to get real time

After alloc integration tests:

TODO


TODO: In my opinion the results don't change iterative library development or CI execution in meaningful ways. For example currently the library doc-tests take ~66s and incremental compilation takes 10+ seconds. However I only have limited knowledge of the various local development workflows that exist, and might be missing one that is significantly impacted by this change.
