#!/usr/bin/env bash

# This script exists as a workaround for https://github.com/rust-lang/rust/issues/142119

set -e

target=$1

if [ -z "$target" ]; then
    echo "Usage:\n\t./stripcore.sh TARGET"
    exit 1
fi

for sym in cbrt ceil copysign fabs fdim floor fmax fmaximum fmin fminimum fmod rint round roundeven sqrt trunc; do \
    objcopy --globalize-symbol=$sym --strip-symbol=$sym "$target"; \
done
