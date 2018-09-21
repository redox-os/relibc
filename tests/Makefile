# Binaries that should generate the same output every time
EXPECT_BINS=\
	args \
	arpainet \
	assert \
	ctype \
	error \
	fcntl/create \
	fcntl/fcntl \
	fnmatch \
	locale \
	math \
	netdb \
	select \
	setjmp \
	signal \
	stdio/all \
	stdio/buffer \
	stdio/freopen \
	stdio/fwrite \
	stdio/getc_unget \
	stdio/printf \
	stdio/rename \
	stdio/scanf \
	stdio/setvbuf \
	stdio/sprintf \
	stdlib/a64l \
	stdlib/atof \
	stdlib/atoi \
	stdlib/env \
	stdlib/mkostemps \
	stdlib/rand \
	stdlib/strtod \
	stdlib/strtol \
	stdlib/strtoul \
	stdlib/system \
	string/mem \
	string/strchr \
	string/strcpy \
	string/strcspn \
	string/strncmp \
	string/strpbrk \
	string/strrchr \
	string/strspn \
	string/strstr \
	string/strtok \
	string/strtok_r \
	strings \
	time/asctime \
	time/gmtime \
	time/localtime \
	time/mktime \
	time/strftime \
	time/time \
	unistd/access \
	unistd/brk \
	unistd/dup \
	unistd/exec \
	unistd/fchdir \
	unistd/fsync \
	unistd/ftruncate \
	unistd/getopt \
	unistd/isatty \
	unistd/pipe \
	unistd/rmdir \
	unistd/sleep \
	unistd/write \
	waitpid \
	wchar/mbrtowc \
	wchar/mbsrtowcs \
	wchar/putwchar \
	wchar/wcrtomb

# Binaries that may generate varied output
BINS=\
	$(EXPECT_BINS) \
	dirent \
	pwd \
	resource/getrusage \
	stdlib/alloc \
	stdlib/bsearch \
	stdlib/mktemp \
	time/gettimeofday \
	time/times \
	unistd/chdir \
	unistd/getcwd \
	unistd/gethostname \
	unistd/getid \
	unistd/link \
	unistd/setid \
	unistd/stat

.PHONY: all $(BINS) clean run expected verify

all: $(BINS)

$(BINS): %: bins/%

clean:
	rm -rf bins gen *.out

run: $(BINS)
	for bin in $^; \
	do \
		echo "# $${bin} #"; \
		"bins/$${bin}" test args || exit $$?; \
	done

expected: $(EXPECT_BINS)
	rm -rf expected
	mkdir -p expected
	for bin in $^; \
	do \
		echo "# $${bin} #"; \
		mkdir -p expected/`dirname $${bin}`; \
		"bins/$${bin}" test args > "expected/$${bin}.stdout" 2> "expected/$${bin}.stderr" || exit $$?; \
	done

verify: $(EXPECT_BINS)
	rm -rf gen
	mkdir -p gen
	for bin in $^; \
	do \
		echo "# $${bin} #"; \
		mkdir -p gen/`dirname $${bin}`; \
		"bins/$${bin}" test args > "gen/$${bin}.stdout" 2> "gen/$${bin}.stderr" || exit $$?; \
		diff -u "gen/$${bin}.stdout" "expected/$${bin}.stdout" || exit $$?; \
		diff -u "gen/$${bin}.stderr" "expected/$${bin}.stderr" || exit $$?; \
	done

CFLAGS=\
	-fno-builtin \
	-fno-stack-protector \
	-Wall \
	-g \
	-nostdinc \
	-nostdlib \
	-isystem ../sysroot/include

HEADLIBS=\
	../sysroot/lib/crt0.o

TAILLIBS=\
	../sysroot/lib/libc.a \
	../sysroot/lib/libm.a

../sysroot:
	make -C .. sysroot

bins/%: %.c ../sysroot
	mkdir -p "$$(dirname "$@")"
	$(CC) $(CFLAGS) $(HEADLIBS) "$<" $(TAILLIBS) -o "$@"
