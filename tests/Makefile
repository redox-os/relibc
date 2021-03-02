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
	ptrace \
	regex \
	select \
	setjmp \
	sigaction \
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
	stdio/printf_space_pad \
	stdio/ungetc_ftell \
	stdio/ungetc_multiple \
	stdio/fscanf_offby1 \
	stdio/fscanf \
	stdio/printf_neg_pad \
	stdlib/a64l \
	stdlib/alloc \
	stdlib/atof \
	stdlib/atoi \
	stdlib/div \
	stdlib/env \
	stdlib/mkostemps \
	stdlib/rand \
	stdlib/rand48 \
	stdlib/random \
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
	sys_mman \
	time/asctime \
	time/constants \
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
	wchar/fwide \
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
	wchar/wcstol \
	wchar/wcscasecmp \
	wchar/wcsncasecmp \
	wctype/towlower \
	wctype/towupper
	# TODO: Fix these
	# mkfifo
	# netdb/netdb \

DYNAMIC_ONLY_NAMES=\
	dlfcn

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

BINS=$(patsubst %,bins_static/%,$(NAMES))
BINS+=$(patsubst %,bins_dynamic/%,$(NAMES))
BINS+=$(patsubst %,bins_dynamic/%,$(DYNAMIC_ONLY_NAMES))
EXPECT_BINS=$(patsubst %,bins_static/%,$(EXPECT_NAMES))
EXPECT_BINS+=$(patsubst %,bins_dynamic/%,$(EXPECT_NAMES))
EXPECT_BINS+=$(patsubst %,bins_dynamic/%,$(DYNAMIC_ONLY_NAMES))

CARGO_TEST?=cargo
TEST_RUNNER?=

.PHONY: all clean run expected verify

all: $(BINS)

clean:
	rm -rf bins_* gen *.out

run: | $(BINS)
	for bin in $(BINS); \
	do \
		echo "# $${bin} #"; \
		"$${bin}" test args || exit $$?; \
	done

expected: | $(EXPECT_BINS)
	rm -rf expected
	mkdir -p expected
	for bin in $(EXPECT_BINS); \
	do \
		echo "# $${bin} #"; \
		mkdir -p expected/`dirname $${bin}`; \
		"$${bin}" test args > "expected/$${bin}.stdout" 2> "expected/$${bin}.stderr" || exit $$?; \
	done

bins_verify/relibc-tests: src/main.rs
	$(CARGO_TEST) build --release --bin relibc-tests --out-dir bins_verify -Z unstable-options

verify: bins_verify/relibc-tests | $(EXPECT_BINS)
	$(TEST_RUNNER) $< $(EXPECT_BINS)

FLAGS=\
	-std=c11 \
	-fno-builtin \
	-fno-stack-protector \
	-Wall \
	-pedantic \
	-g \
	-I .

STATIC_FLAGS=\
	-static

DYNAMIC_FLAGS=\
	-Wl,--enable-new-dtags \
	-Wl,-export-dynamic

../sysroot:
	$(MAKE) -C .. sysroot

NATIVE_RELIBC?=0
ifeq ($(NATIVE_RELIBC),0)
FLAGS+=\
	-nostdinc \
	-nostdlib \
	-isystem ../sysroot/include \
	../sysroot/lib/crt0.o \
	../sysroot/lib/crti.o \
	../sysroot/lib/crtn.o

SYSROOT_LIB=$(shell realpath ../sysroot/lib/)

STATIC_FLAGS+=\
	$(SYSROOT_LIB)/libc.a

DYNAMIC_FLAGS+=\
	-Wl,-dynamic-linker=$(SYSROOT_LIB)/ld64.so.1 \
	-Wl,-rpath=$(SYSROOT_LIB):\$$ORIGIN \
	-L $(SYSROOT_LIB) \
	-lc

DEPS=../sysroot
else
DYNAMIC_FLAGS+=\
	-Wl,-rpath=\$$ORIGIN
endif

bins_static/%: %.c $(DEPS)
	mkdir -p "$$(dirname "$@")"
	$(CC) "$<" -o "$@" $(FLAGS) $(STATIC_FLAGS)

bins_dynamic/%.so: %.c $(DEPS)
	mkdir -p "$$(dirname "$@")"
	$(CC) "$<" -o "$@" -shared -fpic $(FLAGS) $(DYNAMIC_FLAGS)

bins_dynamic/dlfcn: dlfcn.c bins_dynamic/sharedlib.so $(DEPS)
	mkdir -p "$$(dirname "$@")"
	$(CC) "$<" -o "$@" $(FLAGS) $(DYNAMIC_FLAGS)

bins_dynamic/%: %.c $(DEPS)
	mkdir -p "$$(dirname "$@")"
	$(CC) "$<" -o "$@" $(FLAGS) $(DYNAMIC_FLAGS)
