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
	rename \
	rmdir \
	sleep \
	sprintf \
	stdio/fwrite \
	stdio/all \
	stdio/freopen \
	stdlib/bsearch \
	stdlib/strtol \
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
	time

# Binaries that may generate varied output
BINS=\
	$(EXPECT_BINS) \
	alloc \
	chdir \
	getid \
	setid

FORCE:
.PHONY: all clean run gen_expected verify FORCE

all: $(BINS)

clean:
	@$(RM) -v $(BINS) *.out
	@$(RM) -rv gen/
run: $(BINS:%=%.run)
%.run: % FORCE # ignore *.run files that might exist
	@echo "# $* ($<) #"
	@./$* test args || $$?

# Generates files in expected/ directory (if needed)
gen_expected: $(EXPECT_BINS:%=expected/%.stdout) $(EXPECT_BINS:%=expected/%.stderr)
expected/%.stdout epected/%.stderr:
	@mkdir -p $(@D)
	@echo "# $* #"
	@./$* test args > expected/$*.stdout 2> expected/$*.stderr || exit $$?

# Generates files in gen/ directory (if needed) and checks verifies them
# To regenerate those files use `make -B` or `make clean && make verify`
verify: $(EXPECT_BINS:%=gen/%.stdout) $(EXPECT_BINS:%=gen/%.stderr)
gen/%.stdout gen/%.stderr: expected/%.stdout expected/%.stderr
	@mkdir -p $(@D)
	@echo "# $* #"
	@./$* test args > gen/$*.stdout 2> gen/$*.stderr || exit $$?
	@diff -u "gen/$*.stdout" "expected/$*.stdout" || exit $$?
	@diff -u "gen/$*.stderr" "expected/$*.stderr" || exit $$?


CC=gcc

CPPFLAGS=\
	-nostdinc \
	-isystem ../include \
	-isystem ../target/include \
	-isystem ../target/openlibm/include \
	-isystem ../target/openlibm/src

CFLAGS=\
	-Wall \
	-fno-stack-protector

LDFLAGS=\
	-nostdlib

HEADLIBS=\
	../target/debug/crt0.o

TAILLIBS=\
	../target/debug/libc.a \
	../target/openlibm/libopenlibm.a

%: $(HEADLIBS) %.c $(TAILLIBS)
	$(CC) $(CFLAGS) $(CPPFLAGS) $(LDFLAGS) $^ -o $@

# Disable all implicit suffix rules, since we don't want nor need them
.SUFFIXES:
