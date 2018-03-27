.PHONY: all clean fmt test

all: openlibm/libopenlibm.a target/debug/libc.a target/debug/libcrt0.a
	cargo build

clean:
	cargo clean
	make -C openlibm clean
	make -C tests clean

fmt:
	./fmt.sh

test: openlibm/libopenlibm.a
	make -C tests run

target/debug/libc.a:
	cargo build

target/debug/libcrt0.a:
	cargo build --manifest-path src/crt0/Cargo.toml

openlibm/libopenlibm.a:
	CFLAGS=-fno-stack-protector make -C openlibm libopenlibm.a
