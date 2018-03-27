TARGET?=

BUILD=target/debug
ifneq ($(TARGET),)
	BUILD=target/$(TARGET)/debug
	CARGOFLAGS+="--target=$(TARGET)"
	CC=$(TARGET)-gcc
endif

.PHONY: all clean fmt test

all: $(BUILD)/libc.a $(BUILD)/libcrt0.a $(BUILD)/openlibm/libopenlibm.a

clean:
	cargo clean
	make -C tests clean

fmt:
	./fmt.sh

test: all
	make -C tests run

$(BUILD)/libc.a:
	cargo build $(CARGOFLAGS)

$(BUILD)/libcrt0.a:
	cargo build --manifest-path src/crt0/Cargo.toml $(CARGOFLAGS)

$(BUILD)/openlibm: openlibm
	rm -rf $@ $@.partial
	cp -r $< $@.partial
	mv $@.partial $@

$(BUILD)/openlibm/libopenlibm.a: $(BUILD)/openlibm
	CC=$(CC) CFLAGS=-fno-stack-protector make -C $< libopenlibm.a
