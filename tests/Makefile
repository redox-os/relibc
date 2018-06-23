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
	locale \
	math \
	mem \
	pipe \
	printf \
	rename \
	rmdir \
	scanf \
	setjmp \
	sleep \
	sprintf \
	stdio/fwrite \
	stdio/all \
	stdio/freopen \
	stdlib/bsearch \
	stdlib/strtol \
	stdlib/strtoul \
	stdlib/a64l \
	stdlib/rand \
	string/strncmp \
	string/strcspn \
	string/strchr \
	string/strrchr \
	string/strspn \
	string/strstr \
	string/strpbrk \
	string/strtok \
	string/strtok_r \
	unistd/getopt \
	unlink \
	waitpid \
	write \
	time \
	gmtime \
	asctime

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

CFLAGS=\
	-nostdinc \
	-nostdlib \
	-I ../include \
	-I ../target/include \
	-I ../target/openlibm/include \
	-I ../target/openlibm/src \

HEADLIBS=\
	../target/debug/crt0.o

TAILLIBS=\
	../target/debug/libc.a \
	../target/openlibm/libopenlibm.a

%: %.c $(HEADLIBS) $(TAILLIBS)
	gcc -fno-stack-protector -Wall $(CFLAGS) $(HEADLIBS) "$<" $(TAILLIBS) -o "$@"
