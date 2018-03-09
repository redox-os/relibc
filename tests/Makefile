BINS=\
	alloc \
	atof \
	atoi \
	brk \
	args \
	chdir \
	create \
	ctype \
	dup \
	error \
	fchdir \
	fcntl \
	fsync \
	ftruncate \
	getid \
	link \
	math \
	rmdir \
	pipe \
	printf \
	write

all: $(BINS)

clean:
	rm -f $(BINS) *.out

expected: $(BINS)
	mkdir -p expected
	for bin in $(BINS); do "./$${bin}" test args > "expected/$${bin}.stdout" 2> "expected/$${bin}.stderr"; done

run: $(BINS)
	for bin in $(BINS); do echo "# $${bin} #"; "./$${bin}" test args; done

test: $(BINS)
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
