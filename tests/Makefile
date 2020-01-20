# Binaries that should generate the same output every time
EXPECT_NAMES=\
	alloca \
	args \
	arpainet \
	assert \
	constructor \
	ctype \
	destructor \
	dirent/scandir \
	errno \
	error \
	fcntl/create \
	fcntl/fcntl \
	fnmatch \
	futimens \
	libgen \
	locale \
	math \
	netdb/getaddrinfo \
	netdb/netdb \
    ptrace \
	regex \
	select \
	setjmp \
	signal \
	stdio/all \
	stdio/buffer \
	stdio/fgets \
	stdio/fputs \
	stdio/fread \
	stdio/freopen \
	stdio/fseek \
	stdio/fwrite \
	stdio/getc_unget \
	stdio/mutex \
	stdio/popen \
	stdio/printf \
	stdio/rename \
	stdio/scanf \
	stdio/setvbuf \
	stdio/sprintf \
	stdlib/a64l \
	stdlib/alloc \
	stdlib/atof \
	stdlib/atoi \
	stdlib/div \
	stdlib/env \
	stdlib/lcg48 \
	stdlib/mkostemps \
	stdlib/rand \
	stdlib/strtod \
	stdlib/strtol \
	stdlib/strtoul \
	stdlib/system \
	string/mem \
	string/strcat \
	string/strchr \
	string/strcpy \
	string/strcspn \
	string/strlen \
	string/strncmp \
	string/strpbrk \
	string/strrchr \
	string/strspn \
	string/strstr \
	string/strtok \
	string/strtok_r \
	string/strsignal \
	strings \
	time/asctime \
	time/gmtime \
	time/localtime \
	time/macros \
	time/mktime \
	time/strftime \
	time/time \
	tls \
	unistd/access \
	unistd/brk \
	unistd/dup \
	unistd/exec \
	unistd/fchdir \
	unistd/fork \
	unistd/fsync \
	unistd/ftruncate \
	unistd/getopt \
	unistd/getopt_long \
	unistd/pipe \
	unistd/rmdir \
	unistd/sleep \
	unistd/swab \
	unistd/write \
	waitpid \
	wchar/mbrtowc \
	wchar/mbsrtowcs \
	wchar/printf-on-wchars \
	wchar/putwchar \
	wchar/wcrtomb \
	wchar/wcscspn \
	wchar/wcsrchr \
	wchar/wcsstr \
	wchar/wcstod \
	wchar/wcstok \
	wchar/wcstol
	# TODO: Fix these
	# mkfifo

# Binaries that may generate varied output
NAMES=\
	$(EXPECT_NAMES) \
	dirent/main \
	pwd \
	stdio/tempnam \
	stdio/tmpnam \
	stdlib/bsearch \
	stdlib/mktemp \
	stdlib/realpath \
	sys_epoll/epoll \
	sys_utsname/uname \
	time/gettimeofday \
	unistd/chdir \
	unistd/getcwd \
	unistd/gethostname \
	unistd/getid \
	unistd/getpagesize \
	unistd/isatty \
	unistd/link \
	unistd/pathconf \
	unistd/setid \
	unistd/stat \
	unistd/sysconf
#	resource/getrusage
#	time/times

BINS=$(patsubst %,bins/%,$(NAMES))
EXPECT_BINS=$(patsubst %,bins/%,$(EXPECT_NAMES))

TEST_RUNNER?=sh --

.PHONY: all clean run expected verify

all: $(BINS)

clean:
	rm -rf bins gen *.out

run: | $(BINS)
	for name in $(NAMES); \
	do \
		echo "# $${name} #"; \
		"bins/$${name}" test args || exit $$?; \
	done

expected: | $(EXPECT_BINS)
	rm -rf expected
	mkdir -p expected
	for name in $(EXPECT_NAMES); \
	do \
		echo "# $${name} #"; \
		mkdir -p expected/`dirname $${name}`; \
		"bins/$${name}" test args > "expected/$${name}.stdout" 2> "expected/$${name}.stderr" || exit $$?; \
	done

verify: | $(EXPECT_BINS)
	$(TEST_RUNNER) ./verify.sh $(EXPECT_NAMES)

CFLAGS=\
	-std=c11 \
	-fno-builtin \
	-fno-stack-protector \
	-static \
	-Wall \
	-pedantic \
	-g \
	-I .

LIBS=

NATIVE_RELIBC?=0
ifeq ($(NATIVE_RELIBC),0)
CFLAGS+=\
	-nostdinc \
	-nostdlib \
	-isystem ../sysroot/include \
	../sysroot/lib/crt0.o \
	../sysroot/lib/crti.o

LIBS=\
	../sysroot/lib/libc.a \
	../sysroot/lib/crtn.o

../sysroot:
	$(MAKE) -C .. sysroot

bins/%: %.c ../sysroot
	mkdir -p "$$(dirname "$@")"
	$(CC) $(CFLAGS) "$<" $(LIBS) -o "$@"
else
bins/%: %.c
	mkdir -p "$$(dirname "$@")"
	$(CC) $(CFLAGS) "$<" $(LIBS) -o "$@"
endif
