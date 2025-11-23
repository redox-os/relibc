# OpenLibm

[![codecov](https://codecov.io/gh/JuliaMath/openlibm/graph/badge.svg?token=eTAdN7d9cg)](https://codecov.io/gh/JuliaMath/openlibm)

[OpenLibm](https://openlibm.org/) is an effort to have a high quality, portable, standalone
C mathematical library ([`libm`](http://en.wikipedia.org/wiki/libm)).
It can be used standalone in applications and programming language
implementations.

The project was born out of a need to have a good `libm` for the
[Julia programming language](http://www.julialang.org) that worked
consistently across compilers and operating systems, and in 32-bit and
64-bit environments.

## Platform support

OpenLibm builds on Linux, macOS, Windows, FreeBSD, OpenBSD, NetBSD, and
DragonFly BSD.  It builds with both GCC and clang. Although largely
tested and widely used on the x86 and x86-64 architectures, OpenLibm
also supports arm, aarch64, ppc64le, mips, wasm32, riscv, s390(x) and
loongarch64.

## Build instructions

### GNU Make

1. Use GNU Make to build OpenLibm. This is `make` on most systems, but `gmake` on BSDs.
2. Use `make USEGCC=1` to build with GCC. This is the default on
   Linux and Windows.
3. Use `make USECLANG=1` to build with clang. This is the default on OS X, FreeBSD,
   and OpenBSD.
4. Use `make ARCH=wasm32` to build the wasm32 library with clang.
5. Architectures are auto-detected. Use `make ARCH=i386` to force a
   build for i386. Other supported architectures are i486, i586, and
   i686. GCC 4.8 is the minimum requirement for correct codegen on
   older 32-bit architectures.


**Cross Build**
Take `riscv64` as example:
1. install `qemu-riscv64-static`, `gcc-riscv64-linux-gnu`
2. Cross build:
```sh
ARCH=riscv64
TRIPLE=$ARCH-linux-gnu
make ARCH=$ARCH TOOLPREFIX=$TRIPLE-  -j
make -C test ARCH=$ARCH TOOLPREFIX=$TRIPLE-  -j
```

3. Run test with qemu:
```sh
qemu-$ARCH-static -L . -L /usr/$TRIPLE/  test/test-float
qemu-$ARCH-static -L . -L /usr/$TRIPLE/  test/test-double
```


### CMake

1. Create build directory with `mkdir build` and navigate into it with `cd build`.
2. Run CMake to configure project and generate native build system with `cmake /path/to/openlibm/`
or generate project with build system of choice e.g. `cmake /path/to/openlib/ -G "MinGW Makefiles"`.
3. Build with the build system with `cmake --build .`.

Default CMake configuration builds a shared library, this can easily be configured using
[BUILD_SHARED_LIBS](https://cmake.org/cmake/help/latest/variable/BUILD_SHARED_LIBS.html)
configuration option.


## Acknowledgements

PowerPC support for openlibm was graciously sponsored by IBM.
