# -*- mode: makefile-gmake -*-
# vi:ft=make

# Default build rule for any Makefile in this project: all
default: all

OS := $(shell uname)
# Do not forget to bump SOMINOR when changing VERSION,
# and SOMAJOR when breaking ABI in a backward-incompatible way
VERSION = 0.8.0
SOMAJOR = 4
SOMINOR = 0
DESTDIR =
prefix ?= /usr/local
bindir ?= $(prefix)/bin
libdir ?= $(prefix)/lib
includedir ?= $(prefix)/include

ifeq ($(OS), FreeBSD)
pkgconfigdir ?= $(prefix)/libdata/pkgconfig
else
pkgconfigdir ?= $(libdir)/pkgconfig
endif

# Build with Code Coverage
# Only test with Ubuntu + gcc + lcov, may not work for other platform.
# deps:  https://github.com/linux-test-project/lcov
# You don't need to set this flag manually, `make coverage` will do it for you.
# Just Run:  make clean && make coverage -j
CODE_COVERAGE ?= 0

ifneq (,$(findstring $(OS),Darwin FreeBSD OpenBSD))
USEGCC ?= 0
USECLANG ?= 1
endif

ifneq (,$(findstring CLANG,$(MSYSTEM)))
# In MSYS2
USEGCC = 0
USECLANG = 1
endif

ifeq ($(ARCH),wasm32)
USECLANG = 1
USEGCC = 0
TOOLPREFIX = llvm-
endif

USEGCC ?= 1
USECLANG ?= 0
TOOLPREFIX ?=

AR := $(TOOLPREFIX)ar

ifeq ($(USECLANG),1)
USEGCC = 0
CC = clang
CFLAGS_add += -fno-builtin -fno-strict-aliasing
endif

ifeq ($(USEGCC),1)
CC := $(TOOLPREFIX)gcc
CFLAGS_add += -fno-gnu89-inline -fno-builtin
endif

ARCH ?= $(shell $(CC) -dumpmachine | sed "s/\([^-]*\).*$$/\1/")

ifeq ($(ARCH),mingw32)
$(error "the mingw32 compiler you are using fails the openblas testsuite. please see the Julia README.windows.md document for a replacement")
endif

# OS-specific stuff
ifeq ($(ARCH),arm64)
override ARCH := aarch64
endif
ifeq ($(findstring arm,$(ARCH)),arm)
override ARCH := arm
MARCH ?= armv7-a+fp
CFLAGS_add += -mhard-float
endif
ifeq ($(findstring powerpc,$(ARCH)),powerpc)
override ARCH := powerpc
endif
ifeq ($(findstring ppc,$(ARCH)),ppc)
override ARCH := powerpc
endif
ifeq ($(findstring s390,$(ARCH)),s390)
override ARCH := s390
endif
ifneq ($(filter $(ARCH),i386 i486 i586 i686 i387 i487 i587 i687),)
override ARCH := i387
MARCH ?= i686
endif
ifeq ($(ARCH),x86_64)
override ARCH := amd64
endif
ifeq ($(findstring mips,$(ARCH)),mips)
override ARCH := mips
endif
ifeq ($(findstring riscv64,$(ARCH)),riscv64)
override ARCH := riscv64
endif
ifeq ($(findstring loongarch64,$(ARCH)),loongarch64)
override ARCH := loongarch64
endif

# If CFLAGS does not contain a -O optimization flag, default to -O3
ifeq ($(findstring -O,$(CFLAGS)),)
CFLAGS_add += -O3
endif

ifneq (,$(findstring MINGW,$(OS)))
override OS=WINNT
endif

#keep these if statements separate
ifeq ($(OS), WINNT)
SHLIB_EXT = dll
SONAME_FLAG =
shlibdir = $(bindir)
else
ifeq ($(OS), Darwin)
SHLIB_EXT = dylib
SONAME_FLAG = -install_name
else
SHLIB_EXT = so
SONAME_FLAG = -soname
endif
CFLAGS_add += -fPIC
shlibdir = $(libdir)
endif

# Add `-march` to our CFLAGS if it's defined
ifneq ($(MARCH),)
CFLAGS_arch += -march=$(MARCH)
endif

ifeq ($(ARCH),i387)
CFLAGS_arch  += -m32
SFLAGS_arch  += -m32
LDFLAGS_arch += -m32
endif

ifeq ($(ARCH),amd64)
CFLAGS_arch  += -m64
SFLAGS_arch  += -m64
LDFLAGS_arch += -m64
endif

ifeq ($(ARCH),wasm32)
CFLAGS_arch += -ffreestanding -nostdlib -nostdinc --target=wasm32-unknown-unknown
endif

# Add our "arch"-related FLAGS in.  We separate arch-related flags out so that
# we can conveniently get at them for targets that don't want the rest of
# *FLAGS_add, such as the testing Makefile targets
CFLAGS_add  += $(CFLAGS_arch)
SFLAGS_add  += $(SFLAGS_arch)
LDFLAGS_add += $(LDFLAGS_arch)

CFLAGS_add += -std=c99 -Wall -I$(OPENLIBM_HOME) -I$(OPENLIBM_HOME)/include -I$(OPENLIBM_HOME)/$(ARCH) -I$(OPENLIBM_HOME)/src -DASSEMBLER -D__BSD_VISIBLE -Wno-implicit-function-declaration
ifneq ($(filter $(ARCH),i387 amd64 powerpc),)
CFLAGS_add += -I$(OPENLIBM_HOME)/ld80
else
ifneq ($(filter $(ARCH),aarch64 riscv64),)
CFLAGS_add += -I$(OPENLIBM_HOME)/ld128
endif
endif

ifneq ($(filter $(ARCH),i387 amd64),)
# Determines whether `long double` is the same as `double` on this arch.
# linux x86_64, for instance, `long double` is 80 bits wide, whereas on macOS aarch64,
# `long double` is the same as `double`.
LONG_DOUBLE_NOT_DOUBLE := 1
else ifeq ($(ARCH), aarch64)
ifeq ($(filter $(OS),Darwin WINNT),)
LONG_DOUBLE_NOT_DOUBLE := 1
endif
endif

ifeq ($(CODE_COVERAGE),1)
CFLAGS_add  += -g -O0 --coverage
LDFLAGS_add += --coverage
endif # CODE_COVERAGE==1


%.c.o: %.c
	$(CC) $(CPPFLAGS) $(CFLAGS) $(CFLAGS_add) -c $< -o $@

%.S.o: %.S
	$(CC) $(CPPFLAGS) $(SFLAGS) $(SFLAGS_add) $(filter -m% -B% -I% -D%,$(CFLAGS_add)) -c $< -o $@


# Makefile debugging trick:
# call print-VARIABLE to see the runtime value of any variable
print-%:
	@echo '$*=$($*)'
