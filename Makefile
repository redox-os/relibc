TARGET?=

BUILD=target
ifneq ($(TARGET),)
	BUILD="target/$(TARGET)"
	CARGOFLAGS+="--target=$(TARGET)"
endif

ifeq ($(TARGET),aarch64-unknown-linux-gnu)
	CC="aarch64-linux-gnu-gcc"
endif

ifeq ($(TARGET),x86_64-unknown-redox)
	CC="x86_64-unknown-redox-gcc"
endif

SRC=\
	src/* \
	src/*/* \
	src/*/*/* \
	src/*/*/*/*

.PHONY: all clean fmt install libc crt libm test

all: libc libm

clean:
	cargo clean
	make -C tests clean

fmt:
	./fmt.sh

install: all
	mkdir -pv "$(DESTDIR)/lib"
	mkdir -pv "$(DESTDIR)/include"
	cp -rv "include"/* "$(DESTDIR)/include"
	cp -rv "target/include"/* "$(DESTDIR)/include"
	cp -v "$(BUILD)/debug/libc.a" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/debug/crt0.o" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/debug/crti.o" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/debug/crtn.o" "$(DESTDIR)/lib"
	cp -v "$(BUILD)/openlibm/libopenlibm.a" "$(DESTDIR)/lib/libm.a"

libc: $(BUILD)/debug/libc.a crt

crt: $(BUILD)/debug/crt0.o $(BUILD)/debug/crti.o $(BUILD)/debug/crtn.o

libm: $(BUILD)/openlibm/libopenlibm.a

test: all
	make -C tests run

$(BUILD)/debug/libc.a: $(SRC)
	cargo build $(CARGOFLAGS)
	touch $@

$(BUILD)/debug/crt0.o: $(SRC)
	cargo rustc --manifest-path src/crt0/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@
	touch $@

$(BUILD)/debug/crti.o: $(SRC)
	cargo rustc --manifest-path src/crti/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@
	touch $@

$(BUILD)/debug/crtn.o: $(SRC)
	cargo rustc --manifest-path src/crtn/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@
	touch $@

$(BUILD)/release/libc.a: $(SRC)
	cargo build --release $(CARGOFLAGS)
	touch $@

$(BUILD)/release/crt0.o: $(SRC)
	cargo rustc --release --manifest-path src/crt0/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@
	touch $@

$(BUILD)/release/crti.o: $(SRC)
	cargo rustc --release --manifest-path src/crti/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@
	touch $@

$(BUILD)/release/crtn.o: $(SRC)
	cargo rustc --release --manifest-path src/crtn/Cargo.toml $(CARGOFLAGS) -- --emit obj=$@
	touch $@

$(BUILD)/openlibm: openlibm
	rm -rf $@ $@.partial
	mkdir -p $(BUILD)
	cp -r $< $@.partial
	mv $@.partial $@
	touch $@

$(BUILD)/openlibm/libopenlibm.a: $(BUILD)/openlibm
	make CC=$(CC) CFLAGS=-fno-stack-protector -C $< libopenlibm.a
