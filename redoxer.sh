#!/usr/bin/env bash

set -e

if ! which redoxer
then
    cargo install redoxer
fi

if [ ! -d "$HOME/.redoxer/toolchain" ]
then
    redoxer toolchain
fi

export CARGO_TEST="redoxer"
export TEST_RUNNER="redoxer exec --folder . --"
redoxer env make "$@"
