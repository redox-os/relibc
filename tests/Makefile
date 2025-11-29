SKIP_REDOX?=0

# If compiling for Redox, SKIP_REDOX must be 1
ifeq ($(SKIP_REDOX),1)
REDOX_FAILING_TESTS=
else
REDOX_FAILING_TESTS=tls \
	futimens \
	mkfifo \
	stdlib/ptsname \
	sys_mman

endif

# Binaries that should generate the same output every time
EXPECT_NAMES=\
	alloca \
	args \
	arpainet \
	assert \
	constructor \
	ctype \
	crypt/blowfish \
	crypt/md5 \
	crypt/pbkdf2 \
	crypt/scrypt \
	crypt/sha256 \
	crypt/sha512 \
	destructor \
	dirent/fdopendir \
	dirent/scandir \
	endian \
	err \
	errno \
	error \
	fcntl/create \
	fcntl/fcntl \
	fcntl/posix_fallocate \
	features \
	fnmatch \
	glob \
	iso646 \
	libgen \
	locale \
	math \
	netdb/getaddrinfo \
	regex \
	select \
	setjmp \
	sigaction \
	sigaltstack \
	signal \
	stdio/all \
	stdio/buffer \
	stdio/dprintf \
	stdio/fgets \
	stdio/fputs \
	stdio/fread \
	stdio/freopen \
	stdio/fseek \
	stdio/fwrite \
	stdio/getc_unget \
  	stdio/getline \
	stdio/mutex \
	stdio/popen \
	stdio/printf \
	stdio/rename \
	stdio/renameat \
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
	stdlib/getsubopt \
	stdlib/mkostemps \
	stdlib/qsort \
	stdlib/rand \
	stdlib/rand48 \
	stdlib/random \
	stdlib/strtod \
	stdlib/strtol \
	stdlib/strtoul \
	stdlib/system \
	string/mem \
	string/memcpy \
	string/memmem \
	string/strcat \
	string/strchr \
	string/strchrnul \
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
	string/strsep \
	string/strsignal \
	string/stpcpy \
	string/stpncpy \
	strings \
	sys_stat/chmod \
	sys_stat/lstat \
	sys_stat/fstatat \
	sys_syslog/syslog \
	time/asctime \
	time/constants \
	time/gmtime \
	time/localtime \
	time/localtime_r \
	time/macros \
	time/mktime \
	time/strftime \
	time/strptime \
	time/time \
	time/timegm \
	time/tzset \
	unistd/access \
	unistd/brk \
	unistd/constants \
	unistd/confstr \
	unistd/dup \
	unistd/exec \
	unistd/fchdir \
	unistd/fork \
	unistd/fsync \
	unistd/ftruncate \
	unistd/getopt \
	unistd/getopt_long \
	unistd/pipe \
	unistd/readlinkat \
	unistd/rmdir \
	unistd/sleep \
	unistd/swab \
	unistd/write \
	wchar/fgetwc \
	wchar/fwide \
	wchar/mbrtowc \
	wchar/mbsrtowcs \
	wchar/printf-on-wchars \
	wchar/putwchar \
	wchar/wscanf \
	wchar/ungetwc \
	wchar/wprintf \
	wchar/wcrtomb \
	wchar/wcpcpy \
	wchar/wcpncpy \
	wchar/wcschr \
	wchar/wcscspn \
	wchar/wcsdup \
	wchar/wcsrchr \
	wchar/wcsrtombs \
	wchar/wcsstr \
	wchar/wcstod \
	wchar/wcstok \
	wchar/wcstol \
	wchar/wcstoimax \
	wchar/wcstoumax \
	wchar/wcscasecmp \
	wchar/wcsncasecmp \
	wchar/wcsnlen \
	wchar/wcsnrtombs \
	wchar/wcswidth \
	wctype/towlower \
	wctype/towupper \
	mknod \
	mknodat \
	$(REDOX_FAILING_TESTS)
	# TODO: Fix this
	# netdb/netdb \

BUILD?=.

DYNAMIC_ONLY_NAMES=\
	dlfcn \
	dlopen_scopes

# Binaries that may generate varied output
NAMES=\
	$(EXPECT_NAMES) \
	dirent/main \
	dirent/posix_getdents \
	kill-waitpid \
	net/if \
	pty/forkpty \
	psignal \
	pwd \
	sa_restart \
	sigchld \
	signals/kill-self \
	signals/kill0-self \
	signals/kill-invalid \
	signals/kill-permission \
	signals/killpg-esrch \
	signals/killpg-invalid \
	signals/killpg0-self \
	signals/kill-group \
	signals/kill-child \
	signals/killpg-child \
	signals/killpg-self \
	signals/pthread_kill-invalid \
	signals/pthread_kill-child \
	signals/pthread_kill-self \
	signals/pthread_kill0-self \
	signals/raise-compliance \
	signals/sigismember-invalid \
	signals/sigismember-valid \
	signals/sigaddset-add \
	signals/sigdelset-delete \
	signals/signal-h \
	signals/signal-h-2 \
	signals/signal-handle_return \
	signals/signal-handler \
	signals/signal-handler2 \
	signals/signal-ignore \
	signals/signal-invalid \
	signals/signal-uncatchable \
	signals/sigprocmask-3 \
	signals/sigprocmask-4 \
	signals/sigprocmask-5 \
	signals/sigprocmask-6 \
	signals/sigprocmask-7 \
	signals/sigprocmask-8 \
	signals/sigprocmask-9 \
	signals/sigprocmask-10 \
	signals/sigprocmask-11 \
	signals/sigpause-invalid \
	signals/sigpause-revert \
	signals/sigpause-suspend \
	signals/sigprocmask-blocksingle \
	signals/sigrelse-1 \
	signals/sigrelse-2 \
	signals/sigrelse-3 \
	signals/sigset-1 \
	signals/sigset-10 \
	signals/sigset-2 \
	signals/sigset-3 \
	signals/sigset-4 \
	signals/sigset-5 \
	signals/sigset-9 \
	stdio/ctermid \
	stdio/tempnam \
	stdio/tmpnam \
	stdlib/bsearch \
	stdlib/mktemp \
	stdlib/realpath \
	sys_epoll/epoll \
	sys_resource/constants \
	sys_stat/stat \
	sys_statvfs/statvfs \
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
	unistd/sysconf \
	pthread/main \
	pthread/cleanup \
	pthread/exit \
	pthread/extjoin \
	pthread/once \
	pthread/customstack \
	pthread/barrier \
	pthread/rwlock_trylock \
	pthread/rwlock_randtest \
	pthread/mutex_recursive \
	pthread/timeout \
	grp/getgrouplist \
	grp/getgrgid_r \
	grp/getgrnam_r \
	grp/gr_iter \
	waitpid \
	waitpid_multiple \
	sigqueue
#	resource/getrusage
#	time/times

# Tests run with `expect` (require a .c file and an .exp file
# that takes the produced binary as the first argument)
EXPECT_INPUT_NAMES=\
	unistd/getpass

# TODO: Dynamic linking is currently broken on Redox
# TODO: Some tests in NAMES break the kernel
ifeq ($(SKIP_REDOX),0)
BINS=$(patsubst %,$(BUILD)/bins_static/%,$(NAMES))
BINS+=$(patsubst %,$(BUILD)/bins_dynamic/%,$(NAMES))
BINS+=$(patsubst %,$(BUILD)/bins_dynamic/%,$(DYNAMIC_ONLY_NAMES))
EXPECT_BINS+=$(patsubst %,$(BUILD)/bins_dynamic/%,$(EXPECT_NAMES))
EXPECT_BINS+=$(patsubst %,$(BUILD)/bins_dynamic/%,$(DYNAMIC_ONLY_NAMES))
EXPECT_INPUT_BINS=$(patsubst %,$(BUILD)/bins_expect_input/%,$(EXPECT_INPUT_NAMES))
endif

EXPECT_BINS=$(patsubst %,$(BUILD)/bins_static/%,$(EXPECT_NAMES))


CARGO_TEST?=cargo
TEST_RUNNER?=

.PHONY: all clean run expected verify

all: $(BINS)

clean:
	rm -rf bins_* gen *.out

run: | $(BINS) $(EXPECT_INPUT_BINS)
	for bin in $(BINS); \
	do \
		echo "# $${bin} #"; \
		${TEST_RUNNER} "$${bin}" test args || exit $$?; \
	done
	for exp in $(EXPECT_INPUT_BINS); \
	do \
		echo "# expect $$(readlink -e $${exp}.exp) $$(readlink -e $${exp}) #"; \
		expect "$$(readlink -e $${exp}.exp)" "$$(readlink -e $${exp})" test args || exit $$?; \
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
	-Wextra \
	-Werror \
	-Wno-deprecated-declarations \
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
	-lc \
	-fpic

DEPS=../sysroot
else
DYNAMIC_FLAGS+=\
	-Wl,-rpath=\$$ORIGIN
endif

$(BUILD)/bins_static/%: %.c $(DEPS)
	mkdir -p "$$(dirname "$@")"
	$(CC) "$<" -o "$@" $(FLAGS) $(STATIC_FLAGS)

$(BUILD)/bins_expect_input/%: %.c %.exp $(DEPS)
	mkdir -p "$$(dirname "$@")"
	$(CC) "$<" -o "$@" $(FLAGS) $(STATIC_FLAGS)
	cp $(word 2, "$^") $(addsuffix .exp,"$@")

$(BUILD)/bins_dynamic/%.so: %.c $(DEPS)
	mkdir -p "$$(dirname "$@")"
	$(CC) "$<" -o "$@" -shared -fpic $(FLAGS) $(DYNAMIC_FLAGS)

# foobar depends on foo
$(BUILD)/bins_dynamic/libfoobar.so: libfoobar.c $(BUILD)/bins_dynamic/libfoo.so $(DEPS)
	mkdir -p "$$(dirname "$@")"
	$(CC) "$<" -o "$@" -shared -fpic $(FLAGS) $(DYNAMIC_FLAGS) -L $(BUILD)/bins_dynamic -lfoo

$(BUILD)/bins_dynamic/dlfcn: dlfcn.c $(BUILD)/bins_dynamic/sharedlib.so $(DEPS)
	mkdir -p "$$(dirname "$@")"
	$(CC) "$<" -o "$@" $(FLAGS) $(DYNAMIC_FLAGS)

$(BUILD)/bins_dynamic/dlopen_scopes: dlopen_scopes.c $(BUILD)/bins_dynamic/libfoobar.so $(BUILD)/bins_dynamic/libfoo.so $(DEPS)
	mkdir -p "$$(dirname "$@")"
	$(CC) "$<" -o "$@" $(FLAGS) $(DYNAMIC_FLAGS)

$(BUILD)/bins_dynamic/%: %.c $(DEPS)
	mkdir -p "$$(dirname "$@")"
	$(CC) "$<" -o "$@" $(FLAGS) $(DYNAMIC_FLAGS)
