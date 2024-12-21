ifndef TARGET
	export TARGET:=$(shell rustc -Z unstable-options --print target-spec-json | grep llvm-target | cut -d '"' -f4)
endif

CARGO?=cargo
CARGO_TEST?=$(CARGO)
CARGO_COMMON_FLAGS=-Z build-std=core,alloc,compiler_builtins
CARGOFLAGS?=$(CARGO_COMMON_FLAGS)
RUSTCFLAGS?=
export OBJCOPY?=objcopy

BUILD?=$(shell pwd)/target/$(TARGET)
CARGOFLAGS+=--target=$(TARGET)

TARGET_HEADERS?=$(BUILD)/include
export CFLAGS=-I$(TARGET_HEADERS)

HEADERS_UNPARSED=$(shell find src/header -mindepth 1 -maxdepth 1 -type d -not -name "_*" -printf "%f\n")
MHEADERS_UNPARSED=$(shell find src/libm -mindepth 1 -maxdepth 1 -type d -not -name "src" -printf "%f\n")
HEADERS_DEPS=$(shell find src/header -type f \( -name "cbindgen.toml" -o -name "*.rs" \))
#HEADERS=$(patsubst %,%.h,$(subst _,/,$(HEADERS_UNPARSED)))

ifeq ($(TARGET),aarch64-unknown-linux-gnu)
	export CC=aarch64-linux-gnu-gcc
	export LD=aarch64-linux-gnu-ld
	export AR=aarch64-linux-gnu-ar
	export NM=aarch64-linux-gnu-nm
	export OBJCOPY=aarch64-linux-gnu-objcopy
	export CPPFLAGS=
endif

ifeq ($(TARGET),aarch64-unknown-redox)
	export CC=aarch64-unknown-redox-gcc
	export LD=aarch64-unknown-redox-ld
	export AR=aarch64-unknown-redox-ar
	export NM=aarch64-unknown-redox-nm
	export OBJCOPY=aarch64-unknown-redox-objcopy
	export CPPFLAGS=
endif

ifeq ($(TARGET),x86_64-unknown-linux-gnu)
	export CC=x86_64-linux-gnu-gcc
	export LD=x86_64-linux-gnu-ld
	export AR=x86_64-linux-gnu-ar
	export NM=x86_64-linux-gnu-nm
	export OBJCOPY=x86_64-linux-gnu-objcopy
	export CPPFLAGS=
endif

ifeq ($(TARGET),i686-unknown-redox)
	export CC=i686-unknown-redox-gcc
	export LD=i686-unknown-redox-ld
	export AR=i686-unknown-redox-ar
	export NM=i686-unknown-redox-nm
	export OBJCOPY=i686-unknown-redox-objcopy
	export CPPFLAGS=
endif

ifeq ($(TARGET),x86_64-unknown-redox)
	export CC=x86_64-unknown-redox-gcc
	export LD=x86_64-unknown-redox-ld
	export AR=x86_64-unknown-redox-ar
	export NM=x86_64-unknown-redox-nm
	export OBJCOPY=x86_64-unknown-redox-objcopy
	export CPPFLAGS=
endif

ifeq ($(TARGET),riscv64gc-unknown-redox)
	export CC=riscv64-unknown-redox-gcc
	export LD=riscv64-unknown-redox-ld
	export AR=riscv64-unknown-redox-ar
	export NM=riscv64-unknown-redox-nm
	export OBJCOPY=riscv64-unknown-redox-objcopy
	export CPPFLAGS=-march=rv64gc -mabi=lp64d
endif

SRC=\
	Cargo.* \
	$(shell find src -type f)

BUILTINS_VERSION=0.1.70

.PHONY: all clean fmt install install-libs install-headers install-tests libs headers submodules test

all: | headers libs

# TODO: can sed be removed now that cbindgen iirc supports varargs?
headers: $(HEADERS_DEPS)
	rm -rf $(TARGET_HEADERS)
	mkdir -pv $(TARGET_HEADERS)
	cp -rv include/* $(TARGET_HEADERS)
	cp -v "openlibm/include"/*.h $(TARGET_HEADERS)
	cp -v "openlibm/src"/*.h $(TARGET_HEADERS)
	set -e ; \
	for header in $(HEADERS_UNPARSED); do \
		echo "Header $$header"; \
		if test -f "src/header/$$header/cbindgen.toml"; then \
			out=`echo "$$header" | sed 's/_/\//g'`; \
			out="$(TARGET_HEADERS)/$$out.h"; \
			cat "src/header/$$header/cbindgen.toml" cbindgen.globdefs.toml \
				 | cbindgen "src/header/$$header/mod.rs" --config=/dev/stdin --output "$$out"; \
			sed -i "s/va_list __valist/.../g" "$$out"; \
		fi \
	done

	for header in $(MHEADERS_UNPARSED); do \
		out=`echo "$$header" | sed 's/_/\//g'`; \
		out="$(TARGET_HEADERS)/$$out.h"; \
		cbindgen --output "$$out" \
			--config="src/libm/$$header/cbindgen.toml" \
			"src/libm/$$header/mod.rs"; \
		sed -i "s/va_list __valist/.../g" "$$out"; \
	done

clean:
	$(CARGO) clean
	$(MAKE) -C tests clean
	rm -rf sysroot

check:
	$(CARGO) check

fmt:
	./fmt.sh

install-headers: headers libs
	mkdir -pv "$(DESTDIR)/include"
	cp -rv "$(TARGET_HEADERS)"/* "$(DESTDIR)/include"

libs: \
	$(BUILD)/release/libc.a \
	$(BUILD)/release/libc.so \
	$(BUILD)/release/libm.o \
	$(BUILD)/release/crt0.o \
	$(BUILD)/release/crti.o \
	$(BUILD)/release/crtn.o \
	$(BUILD)/release/ld_so

install-libs: headers libs
	mkdir -pv "$(DESTDIR)/lib"
	cp -v "$(BUILD)/release/libc.a" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/release/libc.so" "$(DESTDIR)/lib"
	ln -vnfs libc.so "$(DESTDIR)/lib/libc.so.6"
	cp -v "$(BUILD)/release/crt0.o" "$(DESTDIR)/lib"
	ln -vnfs crt0.o "$(DESTDIR)/lib/crt1.o"
	cp -v "$(BUILD)/release/crti.o" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/release/crtn.o" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/release/ld_so" "$(DESTDIR)/lib/ld64.so.1"
	cp -v "$(BUILD)/release/libm.o" "$(DESTDIR)/lib/"
	# Empty libraries for dl, pthread, and rt
	$(AR) -rcs "$(DESTDIR)/lib/libdl.a"
	$(AR) -rcs "$(DESTDIR)/lib/libpthread.a"
	$(AR) -rcs "$(DESTDIR)/lib/libm.a"
	$(AR) -rcs "$(DESTDIR)/lib/librt.a"

install-tests: tests
	$(MAKE) -C tests
	mkdir -p "$(DESTDIR)/bin/relibc-tests"
	cp -vr tests/bins_static/* "$(DESTDIR)/bin/relibc-tests/"

install: install-headers install-libs

submodules:
	git submodule sync
	git submodule update --init --recursive

sysroot:
	rm -rf $@
	rm -rf $@.partial
	mkdir -p $@.partial
	$(MAKE) install DESTDIR=$@.partial
	mv $@.partial $@
	touch $@

test: sysroot
	# TODO: Fix SIGILL when running cargo test
	# $(CARGO_TEST) test
	$(MAKE) -C tests run
	$(MAKE) -C tests verify

# Debug targets

$(BUILD)/debug/libc.a: $(BUILD)/debug/librelibc.a
	echo "create $@" > "$@.mri"
	for lib in $^; do\
		echo "addlib $$lib" >> "$@.mri"; \
	done
	echo "save" >> "$@.mri"
	echo "end" >> "$@.mri"
	$(AR) -M < "$@.mri"

$(BUILD)/debug/libc.so: $(BUILD)/debug/librelibc.a
	$(CC) -nostdlib -shared -Wl,--allow-multiple-definition -Wl,--whole-archive $^ -Wl,--no-whole-archive -Wl,-soname,libc.so.6 -o $@

$(BUILD)/debug/librelibc.a: $(SRC)
	$(CARGO) rustc $(CARGOFLAGS) -- --emit link=$@ $(RUSTCFLAGS)
	./renamesyms.sh "$@" "$(BUILD)/debug/deps/"
	touch $@

$(BUILD)/debug/libm.o:
	CARGO_INCREMENTAL=0 $(CARGO) rustc --manifest-path src/libm/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort $(RUSTCFLAGS)
	touch $@

$(BUILD)/debug/crt0.o: $(SRC)
	$(CARGO) rustc --manifest-path src/crt0/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort $(RUSTCFLAGS)
	touch $@

$(BUILD)/debug/crti.o: $(SRC)
	$(CARGO) rustc --manifest-path src/crti/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort $(RUSTCFLAGS)
	touch $@

$(BUILD)/debug/crtn.o: $(SRC)
	$(CARGO) rustc --manifest-path src/crtn/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort $(RUSTCFLAGS)
	touch $@

$(BUILD)/debug/ld_so.o: $(SRC)
	$(CARGO) rustc --manifest-path ld_so/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort $(RUSTCFLAGS)
	touch $@

$(BUILD)/debug/ld_so: $(BUILD)/debug/ld_so.o $(BUILD)/debug/crti.o $(BUILD)/debug/libc.a $(BUILD)/debug/crtn.o
	$(LD) --no-relax -T ld_so/ld_script/$(TARGET).ld --allow-multiple-definition --gc-sections $^ -o $@

# Release targets

$(BUILD)/release/libc.a: $(BUILD)/release/librelibc.a
	echo "create $@" > "$@.mri"
	for lib in $^; do\
		echo "addlib $$lib" >> "$@.mri"; \
	done
	echo "save" >> "$@.mri"
	echo "end" >> "$@.mri"
	$(AR) -M < "$@.mri"

$(BUILD)/release/libc.so: $(BUILD)/release/librelibc.a
	$(CC) -nostdlib -shared -Wl,--allow-multiple-definition -Wl,--whole-archive $^ -Wl,--no-whole-archive -Wl,-soname,libc.so.6 -o $@

$(BUILD)/release/librelibc.a: $(SRC)
	$(CARGO) rustc --release $(CARGOFLAGS) -- --emit link=$@ $(RUSTCFLAGS)
	# TODO: Better to only allow a certain whitelisted set of symbols? Perhaps
	# use some cbindgen hook, specify them manually, or grep for #[no_mangle].
	./renamesyms.sh "$@" "$(BUILD)/release/deps/"
	touch $@

$(BUILD)/release/libm.o:
	CARGO_INCREMENTAL=0 $(CARGO) rustc --release --manifest-path src/libm/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort $(RUSTCFLAGS)
	touch $@

$(BUILD)/release/crt0.o: $(SRC)
	$(CARGO) rustc --release --manifest-path src/crt0/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort $(RUSTCFLAGS)
	touch $@

$(BUILD)/release/crti.o: $(SRC)
	$(CARGO) rustc --release --manifest-path src/crti/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort $(RUSTCFLAGS)
	touch $@

$(BUILD)/release/crtn.o: $(SRC)
	$(CARGO) rustc --release --manifest-path src/crtn/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort $(RUSTCFLAGS)
	touch $@

$(BUILD)/release/ld_so.o: $(SRC)
	$(CARGO) rustc --release --manifest-path ld_so/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort $(RUSTCFLAGS)
	touch $@

$(BUILD)/release/ld_so: $(BUILD)/release/ld_so.o $(BUILD)/release/crti.o $(BUILD)/release/libc.a $(BUILD)/release/crtn.o
	$(LD) --no-relax -T ld_so/ld_script/$(TARGET).ld --allow-multiple-definition --gc-sections $^ -o $@

# Other targets

# $(BUILD)/openlibm: openlibm
# 	rm -rf $@ $@.partial
# 	mkdir -p $(BUILD)
# 	cp -r $< $@.partial
# 	mv $@.partial $@
# 	touch $@

# $(BUILD)/openlibm/libopenlibm.a: $(BUILD)/openlibm $(BUILD)/release/librelibc.a
# 	$(MAKE) AR=$(AR) CC=$(CC) LD=$(LD) CPPFLAGS="-fno-stack-protector -I$(shell pwd)/include -I$(TARGET_HEADERS)" -C $< libopenlibm.a
