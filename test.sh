#!/bin/bash
set -ex

cargo build
cargo build --manifest-path src/crt0/Cargo.toml

CFLAGS=-fno-stack-protector make -C openlibm

make -C tests clean
make -C tests run
