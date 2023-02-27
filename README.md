# sort-research-rs

* Is a sort implementation correct?
* Is a sort implementation fast?

This repository contains:

* An exhaustive test suite, including properties not commonly checked or upheld
* An extensive benchmark suite, abstracting over types, patterns and sizes
* A fuzzing harness
* Novel sort implementations ipn_stable and ipn_unstable (Instruction-Parallel-Network)
* Vendored sort implementations (Rust, C++, C), eg. cpp_pdqsort, rust_std_stable

Most tests and benchmarks can be applied to non Rust implementations.
This works by implementing the 5 benchmark types as #[repr(C)] and having
a defined C API that can be implemented, wrapping the C/C++ sorts.

Most functionality is by default disabled via cargo features, see the
Cargo.toml. Some functionality can be enabled or switched by setting environment
variables. See for example benches/bench.rs.

### Installing

[See cargo docs](https://doc.rust-lang.org/cargo/guide/).

Note, you'll need a nightly rust toolchain.

## Running the tests

```
cargo test

cargo miri test

# Might require disabling criterion dependency.
RUSTFLAGS=-Zsanitizer=address cargo t --release
```

## Running the benchmarks

```
cargo bench

cargo bench <sort_name>-<prediction_state>-<type>-<pattern>-<size>

# Eg:
cargo bench rust_std_stable-hot-u64-random-10000
cargo bench hot-u64-random-10000
cargo bench random
```

You can also set a custom regex to filter the benchmarks you want to run:

```
CUSTOM_BENCH_REGEX="std.*i32-random-8$" cargo bench
```

If you want to collect a set of results that can then later be used to create graphs, you can use the `run_benchmarks.py` utility script:

```
CUSTOM_BENCH_REGEX="_stable.*random-" python util/run_benchmarks.py my_test_zen3
```

## Fuzzing

You'll need to install cargo fuzz and cargo afl respectively.
See https://rust-fuzz.github.io/book/introduction.html.

### Fuzzing with libfuzzer

```
cd fuzz
cargo fuzz run libfuzzer_main
```

### Fuzzing with afl

```
cd fuzz-afl
RUSTFLAGS=-Zsanitizer=address cargo afl build --release && cargo afl fuzz -i in -o out -D target/release/afl
```

Adjust `fuzz/fuzz_targets/libfuzzer_main.rs` and
`fuzz/fuzz_targets/libfuzzer_main.rs` respectively to change the fuzz target.
Default `rust_ipn_stable`.

## Contributing

Please respect the [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) when contributing.

## Authors

* **Lukas Bergdoll** - [Voultapher](https://github.com/Voultapher)

See also the list of [contributors](https://github.com/Voultapher/sort-research-rs/contributors)
who participated in this project.

## License

This project is licensed under the Apache License, Version 2.0 -
see the [LICENSE.md](LICENSE.md) file for details.
