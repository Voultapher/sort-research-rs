#!/usr/bin/env bash

set -euo pipefail

rustc --version

cargo clean &> /dev/null

RESULT_TABLE=""

function measure_binary_size_type_impl() {
    BIN_PATH="target/$1/binary-size-measurement"

    cargo build --profile=$1 --quiet --features=$2
    strip "$BIN_PATH"
    BASELINE=$(stat --printf="%s" "$BIN_PATH")

    cargo build --profile=$1 --quiet --features=$2,sort_inst
    strip "$BIN_PATH"
    WITH_SORT=$(stat --printf="%s" "$BIN_PATH")

    BINARY_SIZE=$(($WITH_SORT - $BASELINE))
    RESULT_TABLE="$RESULT_TABLE$1 $3 $BINARY_SIZE\n"
}

function measure_binary_size() {
    RESULT_TABLE="$RESULT_TABLE----------------------------\n"
    measure_binary_size_type_impl "$1" "type_u64" "u64"
    measure_binary_size_type_impl "$1" "type_string" "string"
}

measure_binary_size "release"

set +u
PARAM_1="$1"
set -u

if [ "$PARAM_1" = "all" ]; then
    measure_binary_size "release_lto_thin"
    measure_binary_size "release_lto_thin_opt_level_s"
fi

printf -- "$RESULT_TABLE" | column --table
