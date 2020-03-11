TARGET?=

CARGO?=cargo
CARGOFLAGS=
RUSTCFLAGS=

BUILD=target
ifneq ($(TARGET),)
	BUILD="target/$(TARGET)"
	CARGOFLAGS="--target=$(TARGET)"
endif

ifeq ($(TARGET),aarch64-unknown-linux-gnu)
	export CC=aarch64-linux-gnu-gcc
	export LD=aarch64-linux-gnu-ld
	export AR=aarch64-linux-gnu-ar
endif

ifeq ($(TARGET),aarch64-unknown-redox)
	export CC=aarch64-unknown-redox-gcc
	export LD=aarch64-unknown-redox-ld
	export AR=aarch64-unknown-redox-ar
endif

ifeq ($(TARGET),x86_64-unknown-redox)
	export CC=x86_64-unknown-redox-gcc
	export LD=x86_64-unknown-redox-ld
	export AR=x86_64-unknown-redox-ar
endif

SRC=\
	Cargo.* \
	$(shell find src -type f)

.PHONY: all clean fmt headers install install-headers libs test

all: | headers libs

clean:
	$(CARGO) clean
	$(CARGO) clean --manifest-path cbindgen/Cargo.toml
	$(MAKE) -C tests clean
	rm -rf sysroot

check:
	$(CARGO) check

fmt:
	./fmt.sh

headers: $(BUILD)/include

install-headers: headers
	mkdir -pv "$(DESTDIR)/include"
	cp -rv "include"/* "$(DESTDIR)/include"
	cp -rv "$(BUILD)/include"/* "$(DESTDIR)/include"
	cp -v "openlibm/include"/*.h "$(DESTDIR)/include"
	cp -v "openlibm/src"/*.h "$(DESTDIR)/include"
	cp -v "pthreads-emb/"*.h "$(DESTDIR)/include"

libs: \
	$(BUILD)/release/libc.a \
	$(BUILD)/release/libc.so \
	$(BUILD)/release/crt0.o \
	$(BUILD)/release/crti.o \
	$(BUILD)/release/crtn.o \
	$(BUILD)/release/ld_so

install-libs: libs
	mkdir -pv "$(DESTDIR)/lib"
	cp -v "$(BUILD)/release/libc.a" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/release/libc.so" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/release/crt0.o" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/release/crti.o" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/release/crtn.o" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/release/ld_so" "$(DESTDIR)/lib/ld64.so.1"
	cp -v "$(BUILD)/openlibm/libopenlibm.a" "$(DESTDIR)/lib/libm.a"
	cp -v "$(BUILD)/pthreads-emb/libpthread.a" "$(DESTDIR)/lib/libpthread.a"

install: install-headers install-libs

sysroot: all
	rm -rf $@
	rm -rf $@.partial
	mkdir -p $@.partial
	$(MAKE) install DESTDIR=$@.partial
	mv $@.partial $@
	touch $@

test: sysroot
	$(CARGO) test
	$(MAKE) -C tests verify

# Debug targets

$(BUILD)/debug/libc.a: $(BUILD)/debug/librelibc.a $(BUILD)/pthreads-emb/libpthread.a $(BUILD)/openlibm/libopenlibm.a
	echo "create $@" > "$@.mri"
	for lib in $^; do\
		echo "addlib $$lib" >> "$@.mri"; \
	done
	echo "save" >> "$@.mri"
	echo "end" >> "$@.mri"
	$(AR) -M < "$@.mri"

$(BUILD)/debug/libc.so: $(BUILD)/debug/librelibc.a $(BUILD)/pthreads-emb/libpthread.a $(BUILD)/openlibm/libopenlibm.a
	$(CC) -nostdlib -shared -Wl,--allow-multiple-definition -Wl,--whole-archive $^ -Wl,--no-whole-archive -o $@

$(BUILD)/debug/librelibc.a: $(SRC)
	CARGO_INCREMENTAL=0 $(CARGO) rustc $(CARGOFLAGS) -- --emit link=$@ $(RUSTCFLAGS)
	touch $@

$(BUILD)/debug/crt0.o: $(SRC)
	CARGO_INCREMENTAL=0 $(CARGO) rustc --manifest-path src/crt0/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort $(RUSTCFLAGS)
	touch $@

$(BUILD)/debug/crti.o: $(SRC)
	CARGO_INCREMENTAL=0 $(CARGO) rustc --manifest-path src/crti/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort $(RUSTCFLAGS)
	touch $@

$(BUILD)/debug/crtn.o: $(SRC)
	CARGO_INCREMENTAL=0 $(CARGO) rustc --manifest-path src/crtn/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort $(RUSTCFLAGS)
	touch $@

$(BUILD)/debug/ld_so.o: $(SRC)
	CARGO_INCREMENTAL=0 $(CARGO) rustc --manifest-path src/ld_so/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort $(RUSTCFLAGS)
	touch $@

$(BUILD)/debug/ld_so: $(BUILD)/debug/ld_so.o $(BUILD)/debug/crti.o $(BUILD)/debug/libc.a $(BUILD)/debug/crtn.o
	$(LD) --allow-multiple-definition --gc-sections $^ -o $@

# Release targets

$(BUILD)/release/libc.a: $(BUILD)/release/librelibc.a $(BUILD)/pthreads-emb/libpthread.a $(BUILD)/openlibm/libopenlibm.a
	echo "create $@" > "$@.mri"
	for lib in $^; do\
		echo "addlib $$lib" >> "$@.mri"; \
	done
	echo "save" >> "$@.mri"
	echo "end" >> "$@.mri"
	$(AR) -M < "$@.mri"

$(BUILD)/release/libc.so: $(BUILD)/release/librelibc.a $(BUILD)/pthreads-emb/libpthread.a $(BUILD)/openlibm/libopenlibm.a
	$(CC) -nostdlib -shared -Wl,--allow-multiple-definition -Wl,--whole-archive $^ -Wl,--no-whole-archive -o $@

$(BUILD)/release/librelibc.a: $(SRC)
	CARGO_INCREMENTAL=0 $(CARGO) rustc --release $(CARGOFLAGS) -- --emit link=$@ $(RUSTCFLAGS)
	touch $@

$(BUILD)/release/crt0.o: $(SRC)
	CARGO_INCREMENTAL=0 $(CARGO) rustc --release --manifest-path src/crt0/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort $(RUSTCFLAGS)
	touch $@

$(BUILD)/release/crti.o: $(SRC)
	CARGO_INCREMENTAL=0 $(CARGO) rustc --release --manifest-path src/crti/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort $(RUSTCFLAGS)
	touch $@

$(BUILD)/release/crtn.o: $(SRC)
	CARGO_INCREMENTAL=0 $(CARGO) rustc --release --manifest-path src/crtn/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort $(RUSTCFLAGS)
	touch $@

$(BUILD)/release/ld_so.o: $(SRC)
	CARGO_INCREMENTAL=0 $(CARGO) rustc --release --manifest-path src/ld_so/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ -C panic=abort $(RUSTCFLAGS)
	touch $@

$(BUILD)/release/ld_so: $(BUILD)/release/ld_so.o $(BUILD)/release/crti.o $(BUILD)/release/libc.a $(BUILD)/release/crtn.o
	$(LD) --allow-multiple-definition --gc-sections $^ -o $@

# Other targets

$(BUILD)/include: $(SRC)
	rm -rf $@ $@.partial
	mkdir -p $@.partial
	./include.sh $@.partial
	mv $@.partial $@
	touch $@

$(BUILD)/openlibm: openlibm
	rm -rf $@ $@.partial
	mkdir -p $(BUILD)
	cp -r $< $@.partial
	mv $@.partial $@
	touch $@

$(BUILD)/openlibm/libopenlibm.a: $(BUILD)/openlibm $(BUILD)/include
	$(MAKE) CC=$(CC) CPPFLAGS="-fno-stack-protector -I$(shell pwd)/include -I $(shell pwd)/$(BUILD)/include" -C $< libopenlibm.a

$(BUILD)/pthreads-emb: pthreads-emb
	rm -rf $@ $@.partial
	mkdir -p $(BUILD)
	cp -r $< $@.partial
	mv $@.partial $@
	touch $@

$(BUILD)/pthreads-emb/libpthread.a: $(BUILD)/pthreads-emb $(BUILD)/include
	$(MAKE) CC=$(CC) CFLAGS="-fno-stack-protector -I$(shell pwd)/include -I $(shell pwd)/$(BUILD)/include" -C $< libpthread.a
