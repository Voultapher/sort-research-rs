#!/usr/bin/env bash

set -euxo pipefail

for i in $(seq 1 50);
do
    # BENCH_REGEX="rust_std_unstable-hot-1k-random-" cargo bench --bench bench -- --noplot
    # RUSTFLAGS="-Zsanitizer=address" 
    BENCH_REGEX="rust_std_unstable-hot-1k-random-" cargo bench --target x86_64-unknown-linux-gnu --bench bench -- --noplot
done
