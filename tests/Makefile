# Binaries that should generate the same output every time
EXPECT_BINS=\
	bins/args \
	bins/arpainet \
	bins/assert \
	bins/ctype \
	bins/error \
	bins/fcntl/create \
	bins/fcntl/fcntl \
	bins/fnmatch \
	bins/locale \
	bins/math \
	bins/select \
	bins/setjmp \
	bins/signal \
	bins/stdio/all \
	bins/stdio/setvbuf \
	bins/stdio/freopen \
	bins/stdio/fwrite \
	bins/stdio/getc_unget \
	bins/stdio/printf \
	bins/stdio/rename \
	bins/stdio/scanf \
	bins/stdio/sprintf \
	bins/stdlib/a64l \
	bins/stdlib/atof \
	bins/stdlib/atoi \
	bins/stdlib/env \
	bins/stdlib/mkostemps \
	bins/stdlib/rand \
	bins/stdlib/strtod \
	bins/stdlib/strtol \
	bins/stdlib/strtoul \
	bins/stdlib/system \
	bins/string/mem \
	bins/string/strchr \
	bins/string/strcpy \
	bins/string/strcspn \
	bins/string/strncmp \
	bins/string/strpbrk \
	bins/string/strrchr \
	bins/string/strspn \
	bins/string/strstr \
	bins/string/strtok \
	bins/string/strtok_r \
	bins/strings \
	bins/time/asctime \
	bins/time/gmtime \
	bins/time/localtime \
	bins/time/mktime \
	bins/time/strftime \
	bins/time/time \
	bins/unistd/access \
	bins/unistd/brk \
	bins/unistd/dup \
	bins/unistd/exec \
	bins/unistd/fchdir \
	bins/unistd/fsync \
	bins/unistd/ftruncate \
	bins/unistd/getopt \
	bins/unistd/isatty \
	bins/unistd/pipe \
	bins/unistd/rmdir \
	bins/unistd/sleep \
	bins/unistd/write \
	bins/waitpid \
	bins/wchar/mbrtowc \
	bins/wchar/mbsrtowcs \
	bins/wchar/putwchar \
	bins/wchar/wcrtomb

# Binaries that may generate varied output
BINS=\
	$(EXPECT_BINS) \
	bins/dirent \
	bins/pwd \
	bins/resource/getrusage \
	bins/stdlib/alloc \
	bins/stdlib/bsearch \
	bins/stdlib/mktemp \
	bins/time/gettimeofday \
	bins/time/times \
	bins/unistd/chdir \
	bins/unistd/getcwd \
	bins/unistd/gethostname \
	bins/unistd/getid \
	bins/unistd/link \
	bins/unistd/setid \
	bins/unistd/stat

all: $(BINS)

clean:
	rm -f $(BINS) *.out

run: $(BINS)
	for bin in $^; \
	do \
		echo "# $${bin} #"; \
		"$${bin}" test args || exit $$?; \
	done

expected: $(EXPECT_BINS)
	rm -rf expected
	mkdir -p expected
	for bin in $^; \
	do \
		echo "# $${bin} #"; \
		mkdir -p expected/`dirname $${bin}`; \
		"$${bin}" test args > "expected/$${bin}.stdout" 2> "expected/$${bin}.stderr" || exit $$?; \
	done

verify: $(EXPECT_BINS)
	rm -rf gen
	mkdir -p gen
	for bin in $^; \
	do \
		echo "# $${bin} #"; \
		mkdir -p gen/`dirname $${bin}`; \
		"$${bin}" test args > "gen/$${bin}.stdout" 2> "gen/$${bin}.stderr" || exit $$?; \
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
	-I ../sysroot/include

HEADLIBS=\
	../sysroot/lib/crt0.o

TAILLIBS=\
	../sysroot/lib/libc.a \
	../sysroot/lib/libm.a

../sysroot:
	make -C .. sysroot

bins/%: %.c ../sysroot
	mkdir -p "$$(dirname "$@")"
	gcc $(CFLAGS) $(HEADLIBS) "$<" $(TAILLIBS) -o "$@"
