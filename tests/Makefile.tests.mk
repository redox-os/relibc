
# If compiling for Redox, IS_REDOX must be 1
ifeq ($(IS_REDOX),1)
FAILING_TESTS=
else
# Wrong modified time
FAILING_TESTS := futimens
# Crash, mmap issue
FAILING_TESTS += malloc/usable_size
# Not a FIFO
FAILING_TESTS += mkfifo
# Waitpid had EINTR
FAILING_TESTS += sigchld
# not triggering ERANGE 
FAILING_TESTS += stdlib/ptsname
# Hang
FAILING_TESTS += sys_epoll/epollet
# Kernel hit todo!
FAILING_TESTS += sys_mman/fmap
# Hang
FAILING_TESTS += sys_socket/unixpeername
# Task failed successfully?
FAILING_TESTS += signals/pthread_kill-child
# Got EBADF
FAILING_TESTS += unistd/isatty
# Got EINVAL
FAILING_TESTS += unistd/link
# Signal kept unmasked
FAILING_TESTS += sigqueue
# Unwrap hit, written as TODO
FAILING_TESTS += pthread/customstack
# Returning garbage values
FAILING_TESTS += sys_resource/getrusage
# No times.h header
#FAILING_TESTS += time/times
# Outdated test
#FAILING_TESTS += netdb/netdb

endif

# Binaries that should generate the same output every time
EXPECT_NAMES=\
	alloca \
	arpainet \
	assert \
	ctype \
	crypt/blowfish \
	crypt/md5 \
	crypt/pbkdf2 \
	crypt/scrypt \
	crypt/sha256 \
	crypt/sha512 \
	dirent/fdopendir \
	dirent/scandir \
	endian \
	err \
	errno \
	error \
	fcntl/create \
	fcntl/fcntl \
	fcntl/open \
	fcntl/posix_fallocate \
	features \
	fnmatch \
	glob \
	iso646 \
	libgen \
	locale/duplocale \
	locale/newlocale \
	locale/setlocale \
	math \
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
	stdio/putc_unlocked \
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
	sys_socket/recv \
	sys_socket/recvfrom \
	sys_socket/unixrecv \
	sys_socket/unixrecvfrom \
	sys_socket/unixsocketpair \
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
	unistd/alarm \
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
	mknodat

# Binaries that may generate varied output
VARIED_NAMES=\
	dirent/main \
	dirent/posix_getdents \
	includes \
	kill-waitpid \
	limits \
	net/if \
	netdb/getaddrinfo \
	netdb/getaddrinfo_null \
	pthread/timedwait \
	pty/forkpty \
	psignal \
	pwd \
	sa_restart \
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
	sys_mman/mmap \
	sys_resource/constants \
	sys_socket/getpeername \
	sys_stat/stat \
	sys_statvfs/statvfs \
	sys_utsname/uname \
	time/gettimeofday \
	unistd/chdir \
	unistd/getcwd \
	unistd/gethostname \
	unistd/getid \
	unistd/getpagesize \
	unistd/pathconf \
	unistd/setid \
	unistd/sysconf \
	pthread/main \
	pthread/cleanup \
	pthread/exit \
	pthread/extjoin \
	pthread/once \
	pthread/barrier \
	pthread/rwlock_trylock \
	pthread/rwlock_randtest \
	pthread/mutex_recursive \
	pthread/timeout \
	pthread/tls \
	grp/getgrouplist \
	grp/getgroups \
	grp/getgrgid_r \
	grp/getgrnam_r \
	grp/gr_iter \
	waitpid \
	waitpid_multiple \
	$(FAILING_TESTS)


# Tests that only working with when ld.so exist
DYNAMIC_ONLY_EXPECT_NAMES=\
	dlfcn \
	dlopen_scopes

# Tests that may produce different result when ld.so absent
STATIC_CHECK_EXPECT_NAMES=\
	args \
	constructor \
	destructor \
	stdlib/env \
	unistd/brk \
	tls

# Tests run with `expect` (require a .c file and an .exp file
# that takes the produced binary as the first argument)
EXPECT_INPUT_NAMES=\
	unistd/getpass

# TODO: Dynamic linking doesn't work with NATIVE_LIBC=0
ifeq ($(IS_STATIC),1)
BINS+=$(patsubst %,$(BUILD)/bins_static/%,$(VARIED_NAMES))
EXPECT_BINS+=$(patsubst %,$(BUILD)/bins_static/%,$(EXPECT_NAMES))
EXPECT_BINS+=$(patsubst %,$(BUILD)/bins_static/%,$(STATIC_CHECK_EXPECT_NAMES))
ifeq ($(IS_REDOX),0)
EXPECT_INPUT_BINS=$(patsubst %,$(BUILD)/bins_expect_input/%,$(EXPECT_INPUT_NAMES))
endif
else
BINS+=$(patsubst %,$(BUILD)/bins_dynamic/%,$(VARIED_NAMES))
EXPECT_BINS+=$(patsubst %,$(BUILD)/bins_static/%,$(STATIC_CHECK_EXPECT_NAMES))
EXPECT_BINS+=$(patsubst %,$(BUILD)/bins_dynamic/%,$(EXPECT_NAMES))
EXPECT_BINS+=$(patsubst %,$(BUILD)/bins_dynamic/%,$(STATIC_CHECK_EXPECT_NAMES))
EXPECT_BINS+=$(patsubst %,$(BUILD)/bins_dynamic/%,$(DYNAMIC_ONLY_EXPECT_NAMES))
ifeq ($(IS_REDOX),0)
# TODO: redoxer does not have "expect" binary
EXPECT_INPUT_BINS=$(patsubst %,$(BUILD)/bins_expect_input/%,$(EXPECT_INPUT_NAMES))
endif
endif
