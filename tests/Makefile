include ../config.mk

IS_REDOX?=0
IS_STATIC?=0
BUILD?=./build_$(TARGET)

include ./Makefile.tests.mk

CARGO_TEST?=cargo
TEST_RUNNER?=

.PHONY: all clean run expected verify

all: build

clean:
	rm -rf $(BUILD)/bins_* $(BUILD)/gen $(BUILD)/target

build: $(BUILD)/bins_verify/relibc-tests $(BINS) $(EXPECT_BINS) $(EXPECT_INPUT_BINS) support_build

support_build: $(BUILD)/expected \
	$(BUILD)/Makefile \
	$(BUILD)/Makefile.tests.mk \
	$(BUILD)/example_dir \
	$(BUILD)/select.c \
	$(BUILD)/stdlib/realpath.c \
	$(BUILD)/stdio/fread.in \
	$(BUILD)/stdio/fscanf_offby1.c \
	$(BUILD)/stdio/fscanf.c \
	$(BUILD)/stdio/stdio.in \
	$(BUILD)/stdio/getline.in \
	$(BUILD)/stdio/ungetc_ftell.c \
	$(BUILD)/sys_stat/stat.c \
	$(BUILD)/unistd/link.c \
	$(BUILD)/wchar/fgetwc.in \
	$(BUILD)/wchar/ungetwc.in

run: build
	$(MAKE) run -C $(BUILD)

run-once: $(BUILD)/bins_verify/relibc-tests $(BUILD)/$(TESTBIN) support_build
	$(MAKE) run-once -C $(BUILD) TESTBIN=$(TESTBIN)

$(BUILD)/bins_verify/relibc-tests: src/main.rs
	mkdir -p $(BUILD)/bins_verify
	$(CARGO_TEST) build --release --bin relibc-tests --artifact-dir $(BUILD)/bins_verify --target-dir $(BUILD)/target -Z unstable-options
	rm -rf $(BUILD)/target

$(BUILD)/Makefile: Makefile.run.mk
	cp -a $< "$@"

$(BUILD)/Makefile.tests.mk: Makefile.tests.mk
	cp -a $< "$@"

$(BUILD)/expected: expected
	rm -rf "$@"
	cp -a "$<" $(BUILD)/

$(BUILD)/example_dir: example_dir
	rm -rf "$@"
	cp -a "$<" $(BUILD)/

$(BUILD)/%.in: %.in
	@mkdir -p "$$(dirname "$@")"
	cp "$*.in" "$@"

$(BUILD)/%.c: %.c
	@mkdir -p "$$(dirname "$@")"
	cp "$*.c" "$@"

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

SYSROOT?=$(abspath ../sysroot/$(TARGET)/)
SYSROOT_TARGET?=$(SYSROOT)

$(SYSROOT):
	$(MAKE) -C .. sysroot

NATIVE_LIBC?=0
ifeq ($(NATIVE_LIBC),0)
FLAGS+=\
	-nostdinc \
	-nostdlib \
	-isystem $(SYSROOT)/include \
	$(SYSROOT)/lib/crt0.o \
	$(SYSROOT)/lib/crti.o \
	$(SYSROOT)/lib/crtn.o

ifeq ($(TARGET),aarch64-unknown-redox)
FLAGS+=-mno-outline-atomics
endif

STATIC_FLAGS+=\
	$(SYSROOT)/lib/libc.a

ifeq ($(IS_REDOX),0)
DYNAMIC_FLAGS+=\
	-Wl,-dynamic-linker=$(SYSROOT_TARGET)/$(LD_SO_PATH) \
	-Wl,-rpath=$(SYSROOT_TARGET)/lib:\$$ORIGIN \
	-L $(SYSROOT)/lib \
	-lc \
	-fpic
else
# TODO: Make -dynamic-linker works
DYNAMIC_FLAGS+=\
	-Wl,-rpath=$(SYSROOT_TARGET)/lib:\$$ORIGIN \
	-L $(SYSROOT)/lib \
	-lc \
	-fpic
endif

DEPS=../sysroot/$(TARGET)

else # ifeq ($(NATIVE_LIBC),1)
DYNAMIC_FLAGS+=\
	-Wl,-rpath=\$$ORIGIN
ifeq ($(IS_REDOX),0)
# glibc
FLAGS+=\
	-D_POSIX_C_SOURCE=200809L \
	-D_DEFAULT_SOURCE \
	-D_XOPEN_SOURCE=700 \
	-D_GNU_SOURCE \
	-lcrypt \
	-lm
else
# hosted redox, no extra options needed
endif
endif

$(BUILD)/bins_static/%: %.c $(DEPS)
	mkdir -p "$$(dirname "$@")"
	@$(CC) "$<" -o "$@" $(FLAGS) $(STATIC_FLAGS)

$(BUILD)/bins_expect_input/%: %.c %.exp $(DEPS)
	mkdir -p "$$(dirname "$@")"
	@$(CC) "$<" -o "$@" $(FLAGS) $(STATIC_FLAGS)
	cp "$*.exp" $(addsuffix .exp,"$@")

$(BUILD)/bins_dynamic/%.so: %.c $(DEPS)
	mkdir -p "$$(dirname "$@")"
	@$(CC) "$<" -o "$@" -shared -fpic $(FLAGS) $(DYNAMIC_FLAGS)

# foobar depends on foo
$(BUILD)/bins_dynamic/libfoobar.so: libfoobar.c $(BUILD)/bins_dynamic/libfoo.so $(DEPS)
	mkdir -p "$$(dirname "$@")"
	@$(CC) "$<" -o "$@" -shared -fpic $(FLAGS) $(DYNAMIC_FLAGS) -L $(BUILD)/bins_dynamic -lfoo

$(BUILD)/bins_dynamic/dlfcn: dlfcn.c $(BUILD)/bins_dynamic/sharedlib.so $(DEPS)
	mkdir -p "$$(dirname "$@")"
	@$(CC) "$<" -o "$@" $(FLAGS) $(DYNAMIC_FLAGS)

$(BUILD)/bins_dynamic/dlopen_scopes: dlopen_scopes.c $(BUILD)/bins_dynamic/libfoobar.so $(BUILD)/bins_dynamic/libfoo.so $(DEPS)
	mkdir -p "$$(dirname "$@")"
	@$(CC) "$<" -o "$@" $(FLAGS) $(DYNAMIC_FLAGS)

$(BUILD)/bins_dynamic/%: %.c $(DEPS)
	mkdir -p "$$(dirname "$@")"
	@$(CC) "$<" -o "$@" $(FLAGS) $(DYNAMIC_FLAGS)

$(BUILD)/gen/includes: includes.c $(DEPS)
	rm -rf $@.tmp $@
	mkdir -p $@.tmp
	@$(CC) "$<" -H $(FLAGS) $(DYNAMIC_FLAGS) 2> $@.tmp/graph.txt
	sed -i 's|$(SYSROOT)/include/||g' $@.tmp/graph.txt
	mv $@.tmp $@
