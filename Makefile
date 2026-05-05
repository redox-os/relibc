include config.mk

CARGO?=cargo
CARGO_TEST?=$(CARGO)
CARGO_COMMON_FLAGS=-Z build-std=core,alloc,compiler_builtins
CARGOFLAGS?=$(CARGO_COMMON_FLAGS)
CC_WRAPPER?=
RUSTCFLAGS?=
LINKFLAGS?=-lgcc
USE_RUST_LIBM?=
TESTBIN?=
export OBJCOPY?=objcopy

export CARGO_TARGET_DIR?=$(shell pwd)/target
BUILD?=$(CARGO_TARGET_DIR)/$(TARGET)
CARGOFLAGS+=--target=$(TARGET)
EXCEPT_MATH=-not -name "math"
FEATURE_MATH=
ifneq ($(USE_RUST_LIBM),)
FEATURE_MATH=--features math_libm
EXCEPT_MATH=
endif

TARGET_HEADERS?=$(BUILD)/include
export CFLAGS=-I$(TARGET_HEADERS)

PROFILE?=release

HEADERS_UNPARSED=$(shell find src/header -mindepth 1 -maxdepth 1 -type d -not -name "_*" $(EXCEPT_MATH) -printf "%f\n")
HEADERS_DEPS=$(shell find src/header -type f \( -name "cbindgen.toml" -o -name "*.rs" \))
#HEADERS=$(patsubst %,%.h,$(subst _,/,$(HEADERS_UNPARSED)))

SRC=\
	Cargo.* \
	$(shell find src/ redox-rt/src/ ld_so/src/ redox-ioctl/src/ include/ -type f)

BUILTINS_VERSION=0.1.70

.PHONY: all clean fmt install install-libs install-headers install-tests libs headers submodules test

all: | headers libs

headers: $(HEADERS_DEPS)
	rm -rf $(TARGET_HEADERS)
	mkdir -p $(TARGET_HEADERS)
	cp -r include/* $(TARGET_HEADERS)
ifeq ($(USE_RUST_LIBM),)
	cp "openlibm/include"/*.h $(TARGET_HEADERS)
	cp "openlibm/src"/*.h $(TARGET_HEADERS)
endif
	@set -e ; \
	for header in $(HEADERS_UNPARSED); do \
		if test -f "src/header/$$header/cbindgen.toml"; then \
			echo -e "\033[0;36;49mWriting Header $$header\033[0m"; \
			out=`echo "$$header" | sed 's/_/\//g'`; \
			out="$(TARGET_HEADERS)/$$out.h"; \
			cat "src/header/$$header/cbindgen.toml" cbindgen.globdefs.toml \
				 | cbindgen "src/header/$$header/mod.rs" --config=/dev/stdin --output "$$out"; \
		fi \
	done; echo -e "\033[0;36;49mAll headers written\033[0m";

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
	$(BUILD)/$(PROFILE)/libc.a \
	$(BUILD)/$(PROFILE)/libc.so \
	$(BUILD)/$(PROFILE)/crt0.o \
	$(BUILD)/$(PROFILE)/crti.o \
	$(BUILD)/$(PROFILE)/crtn.o \
	$(BUILD)/$(PROFILE)/ld.so

install-libs: headers libs
	mkdir -pv "$(DESTDIR)/lib"
	cp -v "$(BUILD)/$(PROFILE)/libc.a" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/$(PROFILE)/libc.so" "$(DESTDIR)/lib"
	ln -vnfs libc.so "$(DESTDIR)/lib/libc.so.6"
	cp -v "$(BUILD)/$(PROFILE)/crt0.o" "$(DESTDIR)/lib"
	ln -vnfs crt0.o "$(DESTDIR)/lib/crt1.o"
	ln -vnfs crt0.o "$(DESTDIR)/lib/Scrt1.o"
	cp -v "$(BUILD)/$(PROFILE)/crti.o" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/$(PROFILE)/crtn.o" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/$(PROFILE)/ld.so" "$(DESTDIR)/$(LD_SO_PATH)"
ifeq ($(USE_RUST_LIBM),)
	cp -v "$(BUILD)/openlibm/libopenlibm.a" "$(DESTDIR)/lib/libm.a"
else
	$(AR) -rcs "$(DESTDIR)/lib/libm.a"
endif
	# Empty libraries for dl, pthread, and rt
	$(AR) -rcs "$(DESTDIR)/lib/libdl.a"
	$(AR) -rcs "$(DESTDIR)/lib/libpthread.a"
	$(AR) -rcs "$(DESTDIR)/lib/librt.a"

install-tests: tests
	$(MAKE) -C tests
	mkdir -p "$(DESTDIR)/relibc-tests"
	cp -vr tests/build_$(TARGET)/* "$(DESTDIR)/relibc-tests/"

install: install-headers install-libs

submodules:
	git submodule sync
	git submodule update --init --recursive

sysroot:
	@mkdir -p $@

.PHONY: sysroot/$(TARGET)
sysroot/$(TARGET): | sysroot
	rm -rf $@
	rm -rf $@.partial
	mkdir -p $@.partial
	$(MAKE) install DESTDIR=$(shell pwd)/$@.partial
	mv $@.partial $@
	touch $@

test: sysroot/$(TARGET)
	# TODO: Fix SIGILL when running cargo test
	# $(CARGO_TEST) test
	$(MAKE) -C tests run

test-once: sysroot/$(TARGET)
	$(MAKE) -C tests run-once TESTBIN=$(TESTBIN)


$(BUILD)/$(PROFILE)/libc.so: $(BUILD)/$(PROFILE)/libc.a
	$(CC) -nostdlib \
		-shared \
		-Wl,--gc-sections \
		-Wl,-z,pack-relative-relocs \
		-Wl,--sort-common \
		-Wl,--whole-archive $^ -Wl,--no-whole-archive \
		-Wl,-soname,libc.so.6 \
		$(LINKFLAGS) \
		-o $@

$(BUILD)/$(PROFILE)/ld.so: $(BUILD)/$(PROFILE)/ld_so.o $(BUILD)/$(PROFILE)/libc.a
	# TODO: merge ld.so with libc.so: --dynamic-list=dynamic-list-file
	$(LD) --shared -Bsymbolic --no-relax -T ld_so/ld_script/$(TARGET).ld --gc-sections $^ -o $@

$(BUILD)/$(PROFILE)/libc.a: $(BUILD)/$(PROFILE)/librelibc.a $(BUILD)/openlibm/libopenlibm.a
	echo "create $@" > "$@.mri"
	for lib in $^; do\
		echo "addlib $$lib" >> "$@.mri"; \
	done
	echo "save" >> "$@.mri"
	echo "end" >> "$@.mri"
	$(AR) -M < "$@.mri"

# Debug targets

$(BUILD)/debug/librelibc.a: $(SRC)
	$(CARGO) rustc $(CARGOFLAGS) $(FEATURE_MATH) -- --emit link=$@ -g -C debug-assertions=no $(RUSTCFLAGS)
	./renamesyms.sh "$@" "$(BUILD)/debug/deps/"
	./stripcore.sh "$@"
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
	$(CARGO) rustc --manifest-path ld_so/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort -g -C debug-assertions=no $(RUSTCFLAGS)
	touch $@

# Release targets

$(BUILD)/release/librelibc.a: $(SRC)
	$(CARGO) rustc --release $(CARGOFLAGS) -- --emit link=$@ $(RUSTCFLAGS)
	@# TODO: Better to only allow a certain whitelisted set of symbols? Perhaps
	@# use some cbindgen hook, specify them manually, or grep for #[unsafe(no_mangle)].
	./renamesyms.sh "$@" "$(BUILD)/release/deps/"
	./stripcore.sh "$@"
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

# Other targets

$(BUILD)/openlibm: openlibm
	rm -rf $@ $@.partial
	mkdir -p $(BUILD)
	cp -r $< $@.partial
	mv $@.partial $@
	touch $@

ifeq ($(USE_RUST_LIBM),)
$(BUILD)/openlibm/libopenlibm.a: $(BUILD)/openlibm $(BUILD)/$(PROFILE)/librelibc.a
	$(MAKE) -s AR=$(AR) CC="$(CC_WRAPPER) $(CC)" LD=$(LD) CPPFLAGS="$(CPPFLAGS) -fno-stack-protector -I$(shell pwd)/include -I$(TARGET_HEADERS)" -C $< libopenlibm.a
	./renamesyms.sh "$@" "$(BUILD)/release/deps/"
else
$(BUILD)/openlibm/libopenlibm.a:
	mkdir -p "$(BUILD)/openlibm"
	$(AR) -rcs "$(BUILD)/openlibm/libopenlibm.a"
endif
