# Binaries that should generate the same output every time
EXPECT_BINS=\
	atof \
	atoi \
	brk \
	args \
	create \
	ctype \
	dup \
	error \
	fchdir \
	fcntl \
	fsync \
	ftruncate \
	getid \
	getc_unget \
	link \
	math \
	mem \
	pipe \
	printf \
	rmdir \
	sleep \
	sprintf \
	stdio/fwrite \
	stdio/all \
	stdlib/strtol \
	stdlib/a64l \
	string/strncmp \
	string/strcspn \
	string/strchr \
	string/strrchr \
	string/strspn \
	unlink \
	write

# Binaries that may generate varied output
BINS=\
	$(EXPECT_BINS) \
	alloc \
	chdir \
	getid \
	setid

all: $(BINS)

clean:
	rm -f $(BINS) *.out

run: $(BINS)
	for bin in $^; \
	do \
		echo "# $${bin} #"; \
		"./$${bin}" test args || exit $$?; \
	done

expected: $(EXPECT_BINS)
	rm -rf expected
	mkdir -p expected
	for bin in $^; \
	do \
		echo "# $${bin} #"; \
		mkdir -p expected/`dirname $${bin}`; \
		"./$${bin}" test args > "expected/$${bin}.stdout" 2> "expected/$${bin}.stderr" || exit $$?; \
	done

verify: $(EXPECT_BINS)
	rm -rf gen
	mkdir -p gen
	for bin in $^; \
	do \
		echo "# $${bin} #"; \
		mkdir -p gen/`dirname $${bin}`; \
		"./$${bin}" test args > "gen/$${bin}.stdout" 2> "gen/$${bin}.stderr" || exit $$?; \
		diff -u "gen/$${bin}.stdout" "expected/$${bin}.stdout" || exit $$?; \
		diff -u "gen/$${bin}.stderr" "expected/$${bin}.stderr" || exit $$?; \
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
