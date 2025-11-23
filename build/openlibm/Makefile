OPENLIBM_HOME=$(abspath .)
include ./Make.inc

SUBDIRS = src $(ARCH) bsdsrc
ifeq ($(LONG_DOUBLE_NOT_DOUBLE),1)
# Add ld80 directory on x86 and x64
ifneq ($(filter $(ARCH),i387 amd64),)
SUBDIRS += ld80
else
ifneq ($(filter $(ARCH),aarch64),)
SUBDIRS += ld128
else
endif
endif
endif

define INC_template
TEST=test
override CUR_SRCS = $(1)_SRCS
include $(1)/Make.files
SRCS += $$(addprefix $(1)/,$$($(1)_SRCS))
endef

DIR=test

$(foreach dir,$(SUBDIRS),$(eval $(call INC_template,$(dir))))

DUPLICATE_NAMES = $(filter $(patsubst %.S,%,$($(ARCH)_SRCS)),$(patsubst %.c,%,$(src_SRCS)))
DUPLICATE_SRCS = $(addsuffix .c,$(DUPLICATE_NAMES))

OBJS =  $(patsubst %.f,%.f.o,\
	$(patsubst %.S,%.S.o,\
	$(patsubst %.c,%.c.o,$(filter-out $(addprefix src/,$(DUPLICATE_SRCS)),$(SRCS)))))

# If we're on windows, don't do versioned shared libraries. Also, generate an import library
# for the DLL. If we're on OSX, put the version number before the .dylib.  Otherwise,
# put it after.
ifeq ($(OS), WINNT)
OLM_MAJOR_MINOR_SHLIB_EXT := $(SHLIB_EXT)
LDFLAGS_add += -Wl,--out-implib,libopenlibm.$(OLM_MAJOR_MINOR_SHLIB_EXT).a
else
ifeq ($(OS), Darwin)
OLM_MAJOR_MINOR_SHLIB_EXT := $(SOMAJOR).$(SOMINOR).$(SHLIB_EXT)
OLM_MAJOR_SHLIB_EXT := $(SOMAJOR).$(SHLIB_EXT)
else
OLM_MAJOR_MINOR_SHLIB_EXT := $(SHLIB_EXT).$(SOMAJOR).$(SOMINOR)
OLM_MAJOR_SHLIB_EXT := $(SHLIB_EXT).$(SOMAJOR)
endif
LDFLAGS_add += -Wl,$(SONAME_FLAG),libopenlibm.$(OLM_MAJOR_SHLIB_EXT)
endif

.PHONY: all check test clean distclean \
	install install-static install-shared install-pkgconfig install-headers


OLM_LIBS := libopenlibm.a
ifneq ($(ARCH), wasm32)
OLM_LIBS += libopenlibm.$(OLM_MAJOR_MINOR_SHLIB_EXT)
endif

all : $(OLM_LIBS)

check test: test/test-double test/test-float
	test/test-double
	test/test-float

libopenlibm.a: $(OBJS)
	$(AR) -rcs libopenlibm.a $(OBJS)

libopenlibm.$(OLM_MAJOR_MINOR_SHLIB_EXT): $(OBJS)
	$(CC) -shared $(OBJS) $(LDFLAGS) $(LDFLAGS_add) -o $@
ifneq ($(OS),WINNT)
	ln -sf $@ libopenlibm.$(OLM_MAJOR_SHLIB_EXT)
	ln -sf $@ libopenlibm.$(SHLIB_EXT)
endif

test/test-double: libopenlibm.$(OLM_MAJOR_MINOR_SHLIB_EXT)
	$(MAKE) -C test test-double

test/test-float: libopenlibm.$(OLM_MAJOR_MINOR_SHLIB_EXT)
	$(MAKE) -C test test-float

COVERAGE_DIR:=cov-html
COVERAGE_FILE:=$(COVERAGE_DIR)/libopenlibm.info
# Gen cov report with:  make clean && make coverage -j
coverage: clean-coverage
	$(MAKE) test  CODE_COVERAGE=1
	$(MAKE) gen-cov-report

gen-cov-report:
	-mkdir $(COVERAGE_DIR)
	lcov -d amd64 -d bsdsrc -d ld80 -d src \
		--rc lcov_branch_coverage=1 --capture --output-file $(COVERAGE_FILE)
	genhtml --legend --branch-coverage \
		--title "Openlibm commit `git rev-parse HEAD`" \
		--output-directory $(COVERAGE_DIR)/ \
		$(COVERAGE_FILE)

# Zero coverage statistics and Delete report
clean-coverage:
	-lcov -d amd64 -d bsdsrc -d ld80 -d src --zerocounters
	rm -f ./*/*.gcda
	rm -rf $(COVERAGE_DIR)/

clean: clean-coverage
	rm -f aarch64/*.o amd64/*.o arm/*.o bsdsrc/*.o i387/*.o loongarch64/*.o ld80/*.o ld128/*.o src/*.o powerpc/*.o mips/*.o s390/*.o riscv64/*.o
	rm -f libopenlibm.a libopenlibm.*$(SHLIB_EXT)*
	rm -f ./*/*.gcno
	$(MAKE) -C test clean

openlibm.pc: openlibm.pc.in Make.inc Makefile
	echo "version=${VERSION}" > openlibm.pc
	echo "libdir=$(DESTDIR)$(libdir)" >> openlibm.pc
	echo "includedir=$(DESTDIR)$(includedir)/openlibm" >> openlibm.pc
	cat openlibm.pc.in >> openlibm.pc

install-static: libopenlibm.a
	mkdir -p $(DESTDIR)$(libdir)
	cp -RpP -f libopenlibm.a $(DESTDIR)$(libdir)/

install-shared: libopenlibm.$(OLM_MAJOR_MINOR_SHLIB_EXT)
	mkdir -p $(DESTDIR)$(shlibdir)
ifeq ($(OS), WINNT)
	mkdir -p $(DESTDIR)$(libdir)
	cp -RpP -f libopenlibm.*$(SHLIB_EXT) $(DESTDIR)$(shlibdir)/
	cp -RpP -f libopenlibm.*$(SHLIB_EXT).a $(DESTDIR)$(libdir)/
else
	cp -RpP -f libopenlibm.*$(SHLIB_EXT)* $(DESTDIR)$(shlibdir)/
endif

install-pkgconfig: openlibm.pc
	mkdir -p $(DESTDIR)$(pkgconfigdir)
	cp -RpP -f openlibm.pc $(DESTDIR)$(pkgconfigdir)/

install-headers:
	mkdir -p $(DESTDIR)$(includedir)/openlibm
	cp -RpP -f include/*.h $(DESTDIR)$(includedir)/openlibm
	cp -RpP -f src/*.h $(DESTDIR)$(includedir)/openlibm

install: install-static install-shared install-pkgconfig install-headers
