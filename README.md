# sort-research-rs

Test and benchmark various stable and unstable sort implementations in Rust

For example it contains a partial port of fluxsort https://github.com/scandum/fluxsort from C to Rust.

Notable changes to original:

- Fixes various memory safety bugs (see unguarded_insert or flux_analyze in the original)
- Generalizes the implementation for any slice of T
- Actually stable (see unguarded_insert in the original, maybe I ported it wrong)
- Panic safe, original is only build for Copy types and the comparison function can't panic.

Part of this project is a comprehensive test and benchmark suite