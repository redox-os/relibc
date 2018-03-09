#!/bin/bash
set -ex

./fmt.sh -- --write-mode=diff
./test.sh
cargo build --target=x86_64-unknown-redox
if [ $(arch) == "x86_64" ]
then
    cargo build --target=aarch64-unknown-linux-gnu
else
    cargo build --target=x86_64-unknown-linux-gnu
fi
