ifndef TARGET
	export TARGET:=$(shell rustc -Z unstable-options --print target-spec-json | grep llvm-target | cut -d '"' -f4)
endif

ifeq ($(TARGET),aarch64-unknown-linux-gnu)
	export CC=aarch64-linux-gnu-gcc
	export LD=aarch64-linux-gnu-ld
	export AR=aarch64-linux-gnu-ar
	export NM=aarch64-linux-gnu-nm
	export OBJCOPY=aarch64-linux-gnu-objcopy
	export CPPFLAGS=
	LD_SO_PATH=lib/ld.so.1
endif

ifeq ($(TARGET),aarch64-unknown-redox)
	export CC=aarch64-unknown-redox-gcc
	export LD=aarch64-unknown-redox-ld
	export AR=aarch64-unknown-redox-ar
	export NM=aarch64-unknown-redox-nm
	export OBJCOPY=aarch64-unknown-redox-objcopy
	export CPPFLAGS=
	LD_SO_PATH=lib/ld.so.1
endif

ifeq ($(TARGET),i586-unknown-redox)
	export CC=i586-unknown-redox-gcc
	export LD=i586-unknown-redox-ld
	export AR=i586-unknown-redox-ar
	export NM=i586-unknown-redox-nm
	export OBJCOPY=i586-unknown-redox-objcopy
	export CPPFLAGS=
	LD_SO_PATH=lib/libc.so.1
endif

ifeq ($(TARGET),i686-unknown-redox)
	export CC=i686-unknown-redox-gcc
	export LD=i686-unknown-redox-ld
	export AR=i686-unknown-redox-ar
	export NM=i686-unknown-redox-nm
	export OBJCOPY=i686-unknown-redox-objcopy
	export CPPFLAGS=
	LD_SO_PATH=lib/libc.so.1
endif

ifeq ($(TARGET),x86_64-unknown-linux-gnu)
	export CC=x86_64-linux-gnu-gcc
	export LD=x86_64-linux-gnu-ld
	export AR=x86_64-linux-gnu-ar
	export NM=x86_64-linux-gnu-nm
	export OBJCOPY=objcopy
	export CPPFLAGS=
	LD_SO_PATH=lib/ld64.so.1
endif

ifeq ($(TARGET),x86_64-unknown-redox)
	export CC=x86_64-unknown-redox-gcc
	export LD=x86_64-unknown-redox-ld
	export AR=x86_64-unknown-redox-ar
	export NM=x86_64-unknown-redox-nm
	export OBJCOPY=x86_64-unknown-redox-objcopy
	export CPPFLAGS=
	LD_SO_PATH=lib/ld64.so.1
endif

ifeq ($(TARGET),riscv64gc-unknown-redox)
	export CC=riscv64-unknown-redox-gcc
	export LD=riscv64-unknown-redox-ld
	export AR=riscv64-unknown-redox-ar
	export NM=riscv64-unknown-redox-nm
	export OBJCOPY=riscv64-unknown-redox-objcopy
	export CPPFLAGS=-march=rv64gc -mabi=lp64d
	LD_SO_PATH=lib/ld.so.1
endif
