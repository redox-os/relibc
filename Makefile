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
endif

ifeq ($(TARGET),aarch64-unknown-redox)
	export CC=aarch64-unknown-redox-gcc
endif

ifeq ($(TARGET),x86_64-unknown-redox)
	export CC=x86_64-unknown-redox-gcc
endif

SRC=\
	Cargo.* \
	src/* \
	src/*/* \
	src/*/*/* \
	src/*/*/*/*

.PHONY: all clean fmt include install libc libm test

all: | libc libm libpthread

clean:
	$(CARGO) clean
	make -C tests clean
	rm -rf sysroot

check:
	$(CARGO) check

fmt:
	./fmt.sh

install: all
	mkdir -pv "$(DESTDIR)/include"
	cp -rv "include"/* "$(DESTDIR)/include"
	cp -rv "$(BUILD)/include"/* "$(DESTDIR)/include"
	cp -v "openlibm/include"/*.h "$(DESTDIR)/include"
	cp -v "openlibm/src"/*.h "$(DESTDIR)/include"
	cp -v "pthreads-emb/"*.h "$(DESTDIR)/include"
	mkdir -pv "$(DESTDIR)/lib"
	cp -v "$(BUILD)/release/libc.a" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/release/crt0.o" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/release/crti.o" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/release/crtn.o" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/openlibm/libopenlibm.a" "$(DESTDIR)/lib/libm.a"
	cp -v "$(BUILD)/pthreads-emb/libpthread.a" "$(DESTDIR)/lib/libpthread.a"

libc: $(BUILD)/release/libc.a $(BUILD)/release/crt0.o $(BUILD)/release/crti.o $(BUILD)/release/crtn.o $(BUILD)/include

libm: $(BUILD)/openlibm/libopenlibm.a

libpthread: $(BUILD)/pthreads-emb/libpthread.a

sysroot: all
	rm -rf $@
	rm -rf $@.partial
	mkdir -p $@.partial
	make install DESTDIR=$@.partial
	mv $@.partial $@
	touch $@

test: sysroot
	make -C tests run

$(BUILD)/release/libc.a: $(BUILD)/release/librelibc.a $(BUILD)/pthreads-emb/libpthread.a $(BUILD)/openlibm/libopenlibm.a
	echo "create $@" > "$@.mri"
	for lib in $^; do\
		echo "addlib $$lib" >> "$@.mri"; \
	done
	echo "save" >> "$@.mri"
	echo "end" >> "$@.mri"
	ar -M < "$@.mri"

$(BUILD)/debug/librelibc.a: $(SRC)
	$(CARGO) rustc $(CARGOFLAGS) -- $(RUSTCFLAGS)
	touch $@

$(BUILD)/debug/crt0.o: $(SRC)
	CARGO_INCREMENTAL=0 $(CARGO) rustc --manifest-path src/crt0/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ $(RUSTCFLAGS)
	touch $@

$(BUILD)/debug/crti.o: $(SRC)
	CARGO_INCREMENTAL=0 $(CARGO) rustc --manifest-path src/crti/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ $(RUSTCFLAGS)
	touch $@

$(BUILD)/debug/crtn.o: $(SRC)
	CARGO_INCREMENTAL=0 $(CARGO) rustc --manifest-path src/crtn/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ $(RUSTCFLAGS)
	touch $@

$(BUILD)/release/librelibc.a: $(SRC)
	$(CARGO) rustc --release $(CARGOFLAGS) -- $(RUSTCFLAGS)
	touch $@

$(BUILD)/release/crt0.o: $(SRC)
	CARGO_INCREMENTAL=0 $(CARGO) rustc --release --manifest-path src/crt0/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ $(RUSTCFLAGS)
	touch $@

$(BUILD)/release/crti.o: $(SRC)
	CARGO_INCREMENTAL=0 $(CARGO) rustc --release --manifest-path src/crti/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ $(RUSTCFLAGS)
	touch $@

$(BUILD)/release/crtn.o: $(SRC)
	CARGO_INCREMENTAL=0 $(CARGO) rustc --release --manifest-path src/crtn/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@ $(RUSTCFLAGS)
	touch $@

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
	make CC=$(CC) CPPFLAGS="-fno-stack-protector -I$(shell pwd)/include -I $(shell pwd)/$(BUILD)/include" -C $< libopenlibm.a

$(BUILD)/pthreads-emb: pthreads-emb
	rm -rf $@ $@.partial
	mkdir -p $(BUILD)
	cp -r $< $@.partial
	mv $@.partial $@
	touch $@

$(BUILD)/pthreads-emb/libpthread.a: $(BUILD)/pthreads-emb $(BUILD)/include
	make CC=$(CC) CFLAGS="-fno-stack-protector -I$(shell pwd)/include -I $(shell pwd)/$(BUILD)/include" -C $< libpthread.a
