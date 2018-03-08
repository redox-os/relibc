BINS=\
	alloc \
	brk \
	args \
	chdir \
	create \
	dup \
	error \
	fchdir \
	fsync \
	ftruncate \
	getid \
	link \
	math \
	printf \
	write

all: $(BINS)

clean:
	rm -f $(BINS) *.out

run: $(BINS)
	for bin in $(BINS); do echo "# $${bin} #"; "./$${bin}" test args; done

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
	gcc -fno-stack-protector -Wall $(GCCHEAD) "$<" $(GCCTAIL) -o "$@"
