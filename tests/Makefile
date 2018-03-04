BINS=\
	alloc \
	args \
	create \
	math \
	printf \
	write

all: $(BINS)

clean:
	rm -f $(BINS) *.out

run: $(BINS)
	for bin in $(BINS); do echo "\\033[1m$${bin}\\033[0m"; ./$${bin} test args; done

GCCHEAD=\
	-nostdinc \
	-nostdlib \
	-I ../include \
	-I ../target/include \
	-I ../openlibm/include \
	-I ../openlibm/src \
	../target/debug/libcrt0.a

GCCTAIL=\
	../target/debug/libc.a \
	../openlibm/libopenlibm.a

%: %.c
	gcc -fno-stack-protector $(GCCHEAD) $< $(GCCTAIL) -o $@
