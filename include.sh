#!/usr/bin/env bash

SUPRESS_ALL_THE_ERRORS=yes

set -e

include="$(realpath "$1")"

cargo build --release --manifest-path cbindgen/Cargo.toml
cbindgen="$(realpath cbindgen/target/release/cbindgen)"

if [ "$SUPRESS_ALL_THE_ERRORS" = "yes" ]; then
    echo -e "\e[91mNote: Warnings by cbindgen are suppressed in include.sh.\e[0m"
fi

jobs=()
for config in src/header/*/cbindgen.toml
do
    dir="$(dirname "$config")"
    name="$(basename "$dir")"
    if [ "${name:0:1}" != "_" ]
    then
        header="$include/${name/_//}.h"
        pushd "$dir" > /dev/null
        echo "$dir"
        cbindgen_cmd='"$cbindgen" -c cbindgen.toml -o "$header" mod.rs'
        if [ "$SUPRESS_ALL_THE_ERRORS" = "yes" ]; then
            eval "$cbindgen_cmd" 2>&1 | (grep "^ERROR" -A 3 || true) &
        else
            eval "$cbindgen_cmd" &
        fi
        jobs+=($!)
        popd > /dev/null
    fi
done

for job in "${jobs[@]}"
do
    wait "$job"
done
