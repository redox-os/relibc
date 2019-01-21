#!/usr/bin/env bash

set -e

include="$(realpath "$1")"

cargo build --release --manifest-path cbindgen/Cargo.toml
cbindgen="$(realpath target/release/cbindgen)"

jobs=()
for config in src/header/*/cbindgen.toml
do
    dir="$(dirname "$config")"
    name="$(basename "$dir")"
    if [ "${name:0:1}" != "_" ]
    then
        header="$include/${name/_//}.h"
        pushd "$dir"
        "$cbindgen" -c cbindgen.toml -o "$header" mod.rs &
        jobs+=($!)
        popd
    fi
done

for job in "${jobs[@]}"
do
    wait "$job"
done
