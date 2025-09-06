# sort-research-rs

* Is a sort implementation correct?
* Is a sort implementation fast?

This repository contains:

* An exhaustive test suite, including [properties](writeup/sort_safety/text.md#property-analysis) not commonly checked or upheld
* An extensive benchmark suite, abstracting over types, patterns and sizes
* A fuzzing harness
* A novel sort implementation [ipnsort](ipnsort) (Instruction-Parallel-Network-Sort)
* Vendored sort implementations (Rust, C++, C), e.g. cpp_pdqsort, rust_std_stable
* Various experiments and demonstrations
* Results of the research as [papers](writeup/README.md)

Most tests and benchmarks can be applied to non Rust implementations.
This works by implementing the 5 benchmark types as #[repr(C)] and having
a defined C API that can be implemented, wrapping the C/C++ sorts.

Most functionality is by default disabled via cargo features, see the
Cargo.toml. Some functionality can be enabled or switched by setting environment
variables. See for example benches/bench.rs.

## Research results

* [The unreasonable effectiveness of modern sort algorithms](writeup/unreasonable/text.md)
* [ipnsort: an efficient, generic and robust unstable sort implementation.](writeup/ipnsort_introduction/text.md)
* [driftsort: an efficient, generic and robust stable sort implementation.](writeup/driftsort_introduction/text.md)
* [Fast, small, robust: pick three. Introducing a novel branchless partition implementation.](writeup/lomcyc_partition/text.md)
* [Safety vs Performance. A case study of C, C++ and Rust sort implementations.](writeup/sort_safety/text.md)
* [10~17x faster than what? A performance analysis of Intel's x86-simd-sort (AVX-512)](writeup/intel_avx512/text.md)
* [A performance analysis of glidesort and ipn_stable](writeup/glidesort_perf_analysis/text.md)

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

# Example
cargo bench rust_std_stable-hot-u64-random-10000
cargo bench hot-u64-random-10000
cargo bench random
```

You can also set a custom regex to filter the benchmarks you want to run:

```
BENCH_REGEX="std.*i32-random-8$" cargo bench
```

If you want to collect a set of results that can then later be used to create graphs, you can use the `run_benchmarks.py` utility script:

```
# Will write results to my_test_zen3.json
BENCH_REGEX="_stable.*random-" python util/run_benchmarks.py my_test_zen3
```

To run the `graph_all.py` script to create graphs from this data you need to first install the dependencies as specified in requirements.txt e.g. on Linux:
```
python -m venv venv
source ./venv/bin/active
pip install --requirement util/graph_bench_result/requirements.txt
```

Afterwards, you can use the `graph_all.py` script:
```
python util/graph_bench_result/graph_all.py my_test_zen3.json
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

### Adding a new sort implementation

Please **open an issue before** investing the effort of adding a new sort implementation. The maintainer of this project is currently not interested in building an up-to-date database of all existing sort implementations. Implementations are added based on situational context and relevance to the maintainers' goals. Baseline for all new sort implementations is they must pass all functionality tests, they may pass the [safety tests](https://github.com/Voultapher/sort-research-rs/blob/sort-corectness-writeup/writeup/sort_safety/text.md#property-analysis) but don't have to.

## Authors

* **Lukas Bergdoll** - [Voultapher](https://github.com/Voultapher)

See also the list of [contributors](https://github.com/Voultapher/sort-research-rs/contributors)
who participated in this project.

## License

This project is licensed under the Apache License, Version 2.0 -
see the [LICENSE.md](LICENSE.md) file for details.
