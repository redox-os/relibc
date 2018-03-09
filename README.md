# relibc ![build](https://travis-ci.org/redox-os/relibc.svg?branch=master)
relibc is a portable POSIX C standard library written in Rust. It is under heavy development, and currently supports Redox and Linux.

The motivation for this project is twofold: Reduce issues the redox crew was having with newlib, and create a safer alternative to a C standard library written in C. It is mainly designed to be used under redox, as an alternative to newlib, but it also supports linux syscalls via the [sc](https://crates.io/crates/sc) crate.

## Contributing
Just search for any invocation of the `unimplemented` macro, and hop in! The ci server checks builds for linux and redox, checks formatting (via rustfmt), and runs the test suite. Run `ci.sh` locally to check that your changes will pass travis. Use `fmt.sh` to format your code and `test.sh` to run the C test suite.
