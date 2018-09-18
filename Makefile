TARGET?=

BUILD=target
ifneq ($(TARGET),)
	BUILD="target/$(TARGET)"
	CARGOFLAGS+="--target=$(TARGET)"
endif

ifeq ($(TARGET),aarch64-unknown-linux-gnu)
	CC=aarch64-linux-gnu-gcc
endif

ifeq ($(TARGET),x86_64-unknown-redox)
	CC=x86_64-unknown-redox-gcc
endif

SRC=\
	src/* \
	src/*/* \
	src/*/*/* \
	src/*/*/*/*

.PHONY: all clean fmt include install libc libm test

all: | libc libm

clean:
	cargo clean
	make -C tests clean

check:
	cargo check

fmt:
	./fmt.sh

install: all
	mkdir -pv "$(DESTDIR)/lib"
	mkdir -pv "$(DESTDIR)/include"
	cp -rv "include"/* "$(DESTDIR)/include"
	cp -rv "$(BUILD)/include"/* "$(DESTDIR)/include"
	cp -v "$(BUILD)/release/libc.a" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/release/crt0.o" "$(DESTDIR)/lib"
	cp -rv "openlibm/include"/* "$(DESTDIR)/include"
	cp -rv "openlibm/src"/*.h "$(DESTDIR)/include"
	cp -v "$(BUILD)/openlibm/libopenlibm.a" "$(DESTDIR)/lib/libm.a"

libc: $(BUILD)/release/libc.a $(BUILD)/release/crt0.o $(BUILD)/include

libm: $(BUILD)/openlibm/libopenlibm.a

sysroot: all
	rm -rf $@.partial
	mkdir -p $@.partial
	make install DESTDIR=$@.partial
	mv $@.partial $@
	touch $@

test: all
	make -C tests run

$(BUILD)/debug/libc.a: $(SRC)
	cargo build $(CARGOFLAGS)
	touch $@

$(BUILD)/debug/crt0.o: $(SRC)
	CARGO_INCREMENTAL=0 cargo rustc --manifest-path src/crt0/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@
	touch $@

$(BUILD)/release/libc.a: $(SRC)
	cargo build --release $(CARGOFLAGS)
	touch $@

$(BUILD)/release/crt0.o: $(SRC)
	CARGO_INCREMENTAL=0 cargo rustc --release --manifest-path src/crt0/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@
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
