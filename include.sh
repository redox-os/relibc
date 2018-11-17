#!/usr/bin/env bash

set -ex

include="$(realpath "$1")"
cbindgen="$(realpath cbindgen)"

for config in src/header/*/cbindgen.toml
do
    dir="$(dirname "$config")"
    name="$(basename "$dir")"
    if [ "${name:0:1}" != "_" ]
    then
        header="$include/${name/_//}.h"
        pushd "$dir"
        cargo run --release --manifest-path "$cbindgen/Cargo.toml" -- \
            -c cbindgen.toml -o "$header" mod.rs
        popd
    fi
done
