#!/bin/bash
set -ex

./fmt.sh -- --write-mode=diff
./test.sh
cargo build --target=x86_64-unknown-redox
