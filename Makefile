# Makefile
# Makefile for Relibc
#
# This makefile handles the building of the C library (relibc), the dynamic linker (ld.so),
# and the tests. It supports building mainly for Redox and Linux.

TOML := Cargo.toml
CONFIG := .config
BUILD := build
SRC := src

# CARGO_BUILD_DIR: The directory where cargo puts its output (default: "target")
CARGO_BUILD_DIR := target

# CROSS_TARGET: The variable defining the compilation target architecture.
# If left empty, cargo compiles for the host architecture.
CROSS_TARGET ?=

# Detect the target architecture and OS
CARGO_FLAGS :=
ifneq ($(CROSS_TARGET),)
	CARGO_FLAGS += --target $(CROSS_TARGET)
endif

# Default target is release
PROFILE := release
ifeq ($(PROFILE),release)
	CARGO_FLAGS += --release
endif

# Compiler settings
AR ?= ar
CC ?= gcc
LD ?= ld
NM ?= nm
OBJCOPY ?= objcopy
RANLIB ?= ranlib
STRIP ?= strip

# Retrieve the internal include path for the compiler (GCC/Clang).
# This is crucial for -nostdinc builds to find freestanding headers like <limits.h>, <stdarg.h>.
CC_INTERNAL_INCS := $(shell $(CC) -print-file-name=include)
CC_INTERNAL_INCS_FIXED := $(shell $(CC) -print-file-name=include-fixed)

# Construct the flags. We only add the directory if it exists.
# We intentionally DO NOT include /usr/include here.
CC_NOSTDINC_FLAGS := -nostdinc -I$(CC_INTERNAL_INCS)
ifneq ($(wildcard $(CC_INTERNAL_INCS_FIXED)),)
    CC_NOSTDINC_FLAGS += -I$(CC_INTERNAL_INCS_FIXED)
endif

# Flags for C compilation (used for openlibm and tests)
# Note: We add -Iinclude here to ensure relibc's own headers are found first.
CFLAGS ?= -O2 -g -Wall -Wextra -fPIC
CPPFLAGS ?=

# List of headers to generate
HEADERS := \
	include/alloca.h \
	include/assert.h \
	include/bits/assert.h \
	include/bits/ctype.h \
	include/bits/dirent.h \
	include/bits/elf.h \
	include/bits/errno.h \
	include/bits/fcntl.h \
	include/bits/float.h \
	include/bits/inttypes.h \
	include/bits/limits.h \
	include/bits/locale.h \
	include/bits/malloc.h \
	include/bits/netdb.h \
	include/bits/netinet/in.h \
	include/bits/pthread.h \
	include/bits/sched.h \
	include/bits/signal.h \
	include/bits/stdio.h \
	include/bits/stdlib.h \
	include/bits/sys/ioctl.h \
	include/bits/sys/mman.h \
	include/bits/sys/ptrace.h \
	include/bits/sys/resource.h \
	include/bits/sys/select.h \
	include/bits/sys/socket.h \
	include/bits/sys/stat.h \
	include/bits/sys/time.h \
	include/bits/sys/wait.h \
	include/bits/termios.h \
	include/bits/unistd.h \
	include/bits/wchar.h \
	include/complex.h \
	include/cpio.h \
	include/ctype.h \
	include/dirent.h \
	include/dl-tls.h \
	include/dlfcn.h \
	include/elf.h \
	include/endian.h \
	include/err.h \
	include/errno.h \
	include/fcntl.h \
	include/features.h \
	include/fenv.h \
	include/float.h \
	include/fnmatch.h \
	include/getopt.h \
	include/glob.h \
	include/grp.h \
	include/inttypes.h \
	include/iso646.h \
	include/langinfo.h \
	include/libgen.h \
	include/limits.h \
	include/locale.h \
	include/machine/endian.h \
	include/malloc.h \
	include/math.h \
	include/memory.h \
	include/monetary.h \
	include/net/if.h \
	include/netdb.h \
	include/netinet/in.h \
	include/netinet/in_systm.h \
	include/netinet/ip.h \
	include/netinet/tcp.h \
	include/paths.h \
	include/poll.h \
	include/pthread.h \
	include/pty.h \
	include/pwd.h \
	include/regex.h \
	include/sched.h \
	include/semaphore.h \
	include/setjmp.h \
	include/sgtty.h \
	include/shadow.h \
	include/signal.h \
	include/stdarg.h \
	include/stdatomic.h \
	include/stdbool.h \
	include/stddef.h \
	include/stdint.h \
	include/stdio.h \
	include/stdio_ext.h \
	include/stdlib.h \
	include/stdnoreturn.h \
	include/string.h \
	include/strings.h \
	include/sys/auxv.h \
	include/sys/epoll.h \
	include/sys/file.h \
	include/sys/ioctl.h \
	include/sys/mman.h \
	include/sys/param.h \
	include/sys/poll.h \
	include/sys/procfs.h \
	include/sys/ptrace.h \
	include/sys/queue.h \
	include/sys/random.h \
	include/sys/redox.h \
	include/sys/resource.h \
	include/sys/select.h \
	include/sys/socket.h \
	include/sys/stat.h \
	include/sys/statvfs.h \
	include/sys/syslog.h \
	include/sys/time.h \
	include/sys/timeb.h \
	include/sys/times.h \
	include/sys/types.h \
	include/sys/types_internal.h \
	include/sys/uio.h \
	include/sys/un.h \
	include/sys/user.h \
	include/sys/utsname.h \
	include/sys/wait.h \
	include/sysexits.h \
	include/syslog.h \
	include/tar.h \
	include/termios.h \
	include/time.h \
	include/unistd.h \
	include/utime.h \
	include/utmp.h \
	include/wchar.h \
	include/wctype.h

.PHONY: all clean install install-headers install-libs list-headers test libs

# Default target
all: libs $(BUILD)/crt0.o

# Target specifically for building libraries
libs: $(BUILD)/libc.a $(BUILD)/libc.so

$(BUILD)/include:
	mkdir -p $@

$(BUILD)/openlibm: openlibm
	rm -rf $@
	cp -r $< $@

# Compile openlibm
# We enable -ffreestanding to allow the compiler to provide its own stdint.h if needed,
# BUT we prioritize our generated include/stdint.h via $(BUILD)/include.
# We use absolute paths to avoid sub-make path issues.
$(BUILD)/openlibm/libopenlibm.a: $(BUILD)/include $(BUILD)/openlibm
	$(MAKE) AR=$(AR) CC="$(CC)" LD=$(LD) \
		CPPFLAGS="-ffreestanding -fno-stack-protector -I$(abspath $(BUILD)/include) -I$(abspath $(CARGO_BUILD_DIR)/include) $(CC_NOSTDINC_FLAGS)" \
		CFLAGS="-O3 -fPIC -ffreestanding -fno-stack-protector -I$(abspath $(BUILD)/include) -I$(abspath $(CARGO_BUILD_DIR)/include) $(CC_NOSTDINC_FLAGS)" \
		-C $(BUILD)/openlibm libopenlibm.a

$(BUILD)/libc.a: $(BUILD)/librelibc.a $(BUILD)/openlibm/libopenlibm.a
	echo "create $@" > $(BUILD)/libc.mri
	echo "addlib $(BUILD)/librelibc.a" >> $(BUILD)/libc.mri
	echo "addlib $(BUILD)/openlibm/libopenlibm.a" >> $(BUILD)/libc.mri
	echo "save" >> $(BUILD)/libc.mri
	echo "end" >> $(BUILD)/libc.mri
	$(AR) -M < $(BUILD)/libc.mri

$(BUILD)/libc.so: $(BUILD)/librelibc.a $(BUILD)/openlibm/libopenlibm.a
	$(CC) -nostdlib -shared -Wl,-soname,libc.so -o $@ \
		-Wl,--whole-archive $(BUILD)/librelibc.a $(BUILD)/openlibm/libopenlibm.a -Wl,--no-whole-archive \
		-lgcc

$(BUILD)/librelibc.a: Cargo.toml src/* src/*/* src/*/*/*
	mkdir -p $(BUILD)
	cargo rustc $(CARGO_FLAGS) -- -C soft-float -C code-model=kernel --emit link=$@
	# Verify symbols or post-process if needed (e.g. objcopy)

$(BUILD)/crt0.o: src/crt0/src/lib.rs
	mkdir -p $(BUILD)
	rustc --crate-type object --emit obj=$@ $< $(CARGO_FLAGS)

# Header generation task
headers: $(HEADERS)

# Pattern rule to copy headers if they exist in include/
$(BUILD)/include/%.h: include/%.h
	mkdir -p $(dir $@)
	cp $< $@

install: install-headers install-libs

install-headers: headers
	mkdir -p $(DESTDIR)/include
	cp -r $(BUILD)/include/* $(DESTDIR)/include/

install-libs: libs $(BUILD)/crt0.o
	mkdir -p $(DESTDIR)/lib
	cp $(BUILD)/libc.a $(DESTDIR)/lib/
	cp $(BUILD)/libc.so $(DESTDIR)/lib/
	cp $(BUILD)/crt0.o $(DESTDIR)/lib/

clean:
	cargo clean
	rm -rf $(BUILD)

test: all
	$(MAKE) -C tests all
