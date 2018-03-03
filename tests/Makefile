BINS=\
	create \
	write

all: $(BINS)

clean:
	rm -f $(BINS)

run: $(BINS)
	for bin in $(BINS); do echo "\\033[1m$${bin}\\033[0m"; ./$${bin}; done

%: %.c
	gcc -nostdinc -nostdlib -I ../include -I ../target/include ../target/debug/libcrt0.a $< ../target/debug/libc.a -o $@
