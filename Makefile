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

.PHONY: all clean fmt libc test

all: libc libm

clean:
	cargo clean
	make -C tests clean

fmt:
	./fmt.sh

libc: $(BUILD)/debug/libc.a $(BUILD)/debug/libcrt0.a 

libm: $(BUILD)/openlibm/libopenlibm.a

test: all
	make -C tests run

$(BUILD)/debug/libc.a: src/* src/*/* src/*/*/* src/*/*/*/*
	cargo build $(CARGOFLAGS)

$(BUILD)/debug/libcrt0.a: $(BUILD)/debug/libc.a
	cargo build --manifest-path src/crt0/Cargo.toml $(CARGOFLAGS)

$(BUILD)/release/libc.a: src/* src/*/* src/*/*/* src/*/*/*/*
	cargo build --release $(CARGOFLAGS)

$(BUILD)/release/libcrt0.a: $(BUILD)/release/libc.a
	cargo build --release --manifest-path src/crt0/Cargo.toml $(CARGOFLAGS)

$(BUILD)/openlibm: openlibm
	rm -rf $@ $@.partial
	cp -r $< $@.partial
	mv $@.partial $@

$(BUILD)/openlibm/libopenlibm.a: $(BUILD)/openlibm
	make CC=$(CC) CFLAGS=-fno-stack-protector -C $< libopenlibm.a
