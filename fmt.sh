#!/usr/bin/env bash
ARGS=()

for crate in relibc $(find src -name Cargo.toml | cut -d '/' -f2 | grep -v template)
do
    ARGS+=("--package" "$crate")
done

cargo fmt "${ARGS[@]}" "$@"
