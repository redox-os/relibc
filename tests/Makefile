# Binaries that should generate the same output every time
EXPECT_NAMES=\
	alloca \
	args \
	arpainet \
	assert \
	constructor \
	ctype \
	dirent/scandir \
	errno \
	error \
	fcntl/create \
	fcntl/fcntl \
	fnmatch \
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
	stdio/fputs \
	stdio/fread \
	stdio/fseek \
	stdio/fwrite \
	stdio/mutex \
	stdio/popen \
	stdio/printf \
	stdio/rename \
	stdio/scanf \
	stdio/sprintf \
	stdio/printf_space_pad \
	stdio/ungetc_ftell \
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
	time/gmtime \
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
	wchar/wcstol \
	wchar/wcscasecmp \
	wchar/wcsncasecmp \
	# TODO: Fix these
	# mkfifo
	# netdb/netdb \

# issues with linking stdin, stdout, stderr
STATIC_ONLY_NAMES=\
	futimens \
	stdio/all \
	stdio/buffer \
	stdio/fgets \
	stdio/freopen \
	stdio/getc_unget \
	stdio/setvbuf \
	stdio/ungetc_multiple \
	time/localtime \
	wchar/wcstok \
	wctype/towlower \
	wctype/towupper \
# need to call fini in ld_so's _start
STATIC_ONLY_NAMES+=\
	destructor \
# issues with linking optarg, optind etc.
STATIC_ONLY_NAMES+=\
	unistd/getopt \
	unistd/getopt_long \

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
BINS+=$(patsubst %,bins_static/%,$(STATIC_ONLY_NAMES))
BINS+=$(patsubst %,bins_dynamic/%,$(NAMES))
EXPECT_BINS=$(patsubst %,bins_static/%,$(EXPECT_NAMES))
EXPECT_BINS+=$(patsubst %,bins_static/%,$(STATIC_ONLY_NAMES))
EXPECT_BINS+=$(patsubst %,bins_dynamic/%,$(EXPECT_NAMES))

TEST_RUNNER?=sh --

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

verify: | $(EXPECT_BINS)
	$(TEST_RUNNER) ./verify.sh $(EXPECT_BINS)

FLAGS=\
	-std=c11 \
	-fno-builtin \
	-fno-stack-protector \
	-Wall \
	-pedantic \
	-g \
	-I .

STATIC_FLAGS=\
	../sysroot/lib/libc.a \
	-static

NATIVE_RELIBC?=0
ifeq ($(NATIVE_RELIBC),0)
FLAGS+=\
	-nostdinc \
	-nostdlib \
	-isystem ../sysroot/include \
	../sysroot/lib/crt0.o \
	../sysroot/lib/crti.o \
	../sysroot/lib/crtn.o

../sysroot:
	$(MAKE) -C .. sysroot

bins_static/%: %.c ../sysroot
	mkdir -p "$$(dirname "$@")"
	$(CC) "$<" -o "$@" $(FLAGS) $(STATIC_FLAGS)

SYSROOT_LIB=$(shell realpath ../sysroot/lib/)

DYNAMIC_FLAGS=\
	-Wl,-dynamic-linker=$(SYSROOT_LIB)/ld64.so.1 \
	-Wl,--enable-new-dtags \
	-Wl,-rpath=$(SYSROOT_LIB) \
	-L $(SYSROOT_LIB) \
	-lc

bins_dynamic/%: %.c ../sysroot
	mkdir -p "$$(dirname "$@")"
	$(CC) "$<" -o "$@" $(FLAGS) $(DYNAMIC_FLAGS)
else
bins_static/%: %.c
	mkdir -p "$$(dirname "$@")"
	$(CC) "$<" -o "$@" $(FLAGS) $(STATIC_FLAGS)

bins_dynamic/%: %.c
	mkdir -p "$$(dirname "$@")"
	$(CC) "$<" -o "$@" $(FLAGS)
endif
