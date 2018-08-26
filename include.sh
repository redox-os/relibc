#!/usr/bin/env bash

set -ex

include="$(realpath "$1")"
cbindgen="$(realpath cbindgen)"

for config in src/header/*/cbindgen.toml
do
    dir="$(dirname "$config")"
    name="$(basename "$dir")"
    pushd "$dir"
    cargo run --release --manifest-path "$cbindgen/Cargo.toml" -- \
        -c cbindgen.toml -o "$include/$name.h" mod.rs
    popd
done
