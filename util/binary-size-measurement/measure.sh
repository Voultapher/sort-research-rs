#!/usr/bin/env bash

set -u

rustc --version

RESULT_TABLE=""

function measure_binary_size_type_impl() {
    mkdir -p out
    BIN_PATH="target/$1/binary-size-measurement"

    tmpfile=$(mktemp)

    cargo bloat --profile=$1 --features $2 --no-relative-size -n 0 --message-format json > out/baseline_$2_$1.json 2> "$tmpfile"

    if [ $? -ne 0 ]; then
        cat "$tmpfile"
    fi

    cargo bloat --profile=$1 --features $2,sort_inst --no-relative-size -n 0 --message-format json > out/with_sort_$2_$1.json 2> "$tmpfile"

    if [ $? -ne 0 ]; then
        cat "$tmpfile"
    fi

    rm "$tmpfile"

    BINARY_SIZE=$(python eval_bloat.py out/baseline_$2_$1.json out/with_sort_$2_$1.json)
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
