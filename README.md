# relibc ![build](https://travis-ci.org/redox-os/relibc.svg?branch=master)
relibc is a portable POSIX C standard library written in Rust. It is under heavy development, and currently supports Redox and Linux.

The motivation for this project is twofold: Reduce issues the redox crew was having with newlib, and create a safer alternative to a C standard library written in C. It is mainly designed to be used under redox, as an alternative to newlib, but it also supports linux syscalls via the [sc](https://crates.io/crates/sc) crate.

### [Contributing](CONTRIBUTING.md)

## Supported OSes

 - Redox OS
 - Linux

## Supported architectures

 - x86\_64
 - Aarch64
