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
	pipe \
	printf \
	rmdir \
	setid \
	sleep \
	sprintf \
  	stdlib/strtol \
	unlink \
	write

all: $(BINS)

clean:
	rm -f $(BINS) *.out

run: $(BINS)
	for bin in $(BINS); \
	do \
		echo "# $${bin} #"; \
		"./$${bin}" test args; \
	done

expected: $(BINS)
	rm -rf expected
	mkdir -p expected
	for bin in $(BINS); \
	do \
		echo "# $${bin} #"; \
		mkdir -p expected/`dirname $${bin}`; \
		"./$${bin}" test args > "expected/$${bin}.stdout" 2> "expected/$${bin}.stderr"; \
	done

verify: $(BINS)
	rm -rf gen
	mkdir -p gen
	for bin in $(BINS); \
	do \
		echo "# $${bin} #"; \
		mkdir -p gen/`dirname $${bin}`; \
		"./$${bin}" test args > "gen/$${bin}.stdout" 2> "gen/$${bin}.stderr"; \
		diff -u "gen/$${bin}.stdout" "expected/$${bin}.stdout"; \
		diff -u "gen/$${bin}.stderr" "expected/$${bin}.stderr"; \
	done

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
