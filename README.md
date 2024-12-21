# Redox C Library (relibc)

relibc is a portable POSIX C standard library written in Rust and is under heavy development.

The motivation for this project is twofold: Reduce issues that the Redox developers were having with [newlib](https://sourceware.org/newlib/), and create a safer alternative to a C standard library written in C. It is mainly designed to be used under Redox, as an alternative to newlib, but it also supports Linux system calls via the [sc](https://crates.io/crates/sc) crate.

Currently Redox and Linux are supported.

## redox-rt

redox-rt is our equivalent for [vDSO](https://en.wikipedia.org/wiki/VDSO) from Linux.

## Repository Layout

- `include` - Header files (mostly macros and variadic functions `cbindgen` can't generate)
- `src` - Source files
- `src/c` - C code
- `src/crt0` - Runtime code
- `src/crti` - Runtime code
- `src/crtn` - Runtime code
- `src/header` - Header files implementation
- `src/header/*` - Each folder has a `cbindgen.toml` file, it generates a C-to-Rust interface and header files
- `src/ld_so` - Dynamic loader code
- `src/platform` - Platform-specific and common code
- `src/platform/redox` - Redox-specific code
- `src/platform/linux` - Linux-specific code
- `src/pthread` - pthread implementation
- `src/sync` - Synchronization primitives
- `tests` - C tests (each MR needs to give success in all of them)

## Download the sources

To download the relibc sources run the following command:

```sh
git clone --recursive https://gitlab.redox-os.org/redox-os/relibc
```

## Build Instructions

To build relibc out of the Redox build system, do the following steps:

### Dependencies

- Install `cbindgen`

```sh
cargo install cbindgen
```

#### Install the `expect` tool

- Debian, Ubuntu and PopOS:

```sh
sudo apt install expect
```

- Fedora:

```sh
sudo dnf install expect
```

- Arch Linux:

```sh
sudo pacman -S expect
```

### Build Relibc

To build the relibc library objects, run the following command:

```sh
make all
```

- Clean old library objects and tests

```sh
make clean
```

## Build relibc inside the Redox build system

Inside of your Redox build system, run:

```sh
make prefix
```

If you need to rebuild `relibc` for testing a Cookbook recipe, run:

```sh
touch relibc
make prefix r.recipe-name
```

Touching (changing the "last modified time" of) the `relibc` folder is needed to trigger recompilation for `make prefix`. Replace `recipe-name` with your desired recipe name.

Note: Do not edit `relibc` inside `prefix` folder! Do your work on `relibc` folder directly inside your Redox build system instead.

## Tests

This section explain how to build and run the tests.

### Build

To build the tests run `make all` on the `tests` folder, it will store the executables at `tests/bins_static`

If you did changes to your tests, run `make clean all` to rebuild the executables.

### Redox OS Testing

To test on Redox do the following steps:

- Add the `relibc-tests` recipe on your filesystem configuration at `config/your-cpu/your-config.toml` (generally `desktop.toml`)
- Run the following commands to rebuild relibc with your changes, update the `relibc-tests` recipe and update your QEMU image:

```sh
touch relibc
```

```sh
make prefix cr.relibc-tests image
```

- Run the tests

```sh
/usr/share/relibc-tests/bins_static/test-name
```

### Linux Testing

Run `make test` on the relibc directory.

If you want to run one test, run the following command:

```sh
tests/bins_static/test-name
```

## Issues

#### I'm building for my own platform which I run, and am getting `x86_64-linux-gnu-ar: command not found` (or similar)

The Makefile expects GNU compiler tools prefixed with the platform specifier, as would be present when you installed a cross compiler. Since you are building for your own platform, some Linux distributions (like Manjaro) don't install/symlink the prefixed executables.

An easy fix would be to replace the corresponding lines in the Makefile, e.g.

```diff
 ifeq ($(TARGET),x86_64-unknown-linux-gnu)
        export CC=x86_64-linux-gnu-gcc
-       export LD=x86_64-linux-gnu-ld
-       export AR=x86_64-linux-gnu-ar
+       export LD=ld
+       export AR=ar
        export OBJCOPY=x86_64-linux-gnu-objcopy
 endif
```

## Contributing

Before starting to contribute, read [this](CONTRIBUTING.md) document.

## Supported OSes

- Redox OS
- Linux

## Supported architectures

- i686 (Intel/AMD)
- x86_64 (Intel/AMD)
- Aarch64 (ARM64)
