#!/usr/bin/env bash

set -euxo pipefail

BENCH_REGEX="(hoare|lomuto).*(-u64-|-random-)" python3 util/run_benchmarks.py $1
