# relibc ![build](https://travis-ci.org/redox-os/relibc.svg?branch=master)
relibc is a portable POSIX C standard library written in Rust. It is under heavy development, and currently supports Redox, Linux and DragonOS.

The motivation for this project is twofold: Reduce issues the redox crew was having with newlib, and create a safer alternative to a C standard library written in C. It is mainly designed to be used under redox, as an alternative to newlib, but it also supports linux syscalls via the [sc](https://crates.io/crates/sc) crate.

## Building
Just run `make all`.

### Build for DragonOS

You can follow the instructions to build relibc for DragonOS:

```bash
mkdir -p sysroot/usr
make -j $(nproc) && DESTDIR=sysroot/usr make install -j $(nproc)
```

### Issues
#### I'm building for my own platform which I run, and am getting `x86_64-linux-gnu-ar: command not found` (or similar)
The Makefile expects the gnu compiler tools prefixed with the platform specifier, as would be present when you'd install a cross compiler. Since you are building for your own platform, some distros (like Manjaro) don't install/symlink the prefixed executables.
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

### [Contributing](CONTRIBUTING.md)

## Supported OSes

 - Redox OS
 - Linux
 - DragonOS

## Supported architectures

 - x86\_64
 - Aarch64
