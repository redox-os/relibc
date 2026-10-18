#include <assert.h>
#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <threads.h>

#define SHARED_LIB "sharedlib.so"
#define SHARED_LIB_CPP "sharedlib_cpp.so"

int add(int a, int b) { return a + b; }

void test_dlopen_null(int rtld) {
    void *handle = dlopen(NULL, rtld);
    if (!handle) {
        printf("dlopen(NULL) failed: %s\n", dlerror());
        exit(1);
    }

    int (*f)(int, int);
    *(void **)(&f) = dlsym(handle, "add");

    if (!f) {
        printf("dlsym(handle, add) failed: %s\n", dlerror());
        exit(2);
    }
    int a = 22;
    int b = 33;
    printf("add(%d, %d) = %d\n", a, b, f(a, b));
    dlclose(handle);
}

void test_dlopen_libc(int rtld) {
    void *handle = dlopen("libc.so.6", rtld);
    if (!handle) {
        printf("dlopen(libc.so.6) failed\n");
        exit(1);
    }

    int (*f)(const char *);
    *(void **)(&f) = dlsym(handle, "puts");

    if (!f) {
        printf("dlsym(handle, puts) failed\n");
        exit(2);
    }
    f("puts from dlopened libc");
    dlclose(handle);
}

void test_dlsym_function(char *lib, int rtld) {
    void *handle = dlopen(lib, rtld);
    if (!handle) {
        printf("dlopen(%s) failed: %s\n", lib, dlerror());
        exit(1);
    }

    void (*f)();
    *(void **)(&f) = dlsym(handle, "print");

    if (!f) {
        printf("dlsym(handle, print) failed\n");
        exit(2);
    }
    f();
    dlclose(handle);
}

void test_dlsym_global_var(char *lib, int rtld) {
    void *handle = dlopen(lib, rtld);
    if (!handle) {
        printf("dlopen(%s) failed: %s\n", lib, dlerror());
        exit(1);
    }
    int *global_var = dlsym(handle, "global_var");
    if (!global_var) {
        printf("dlsym(handle, global_var) failed\n");
        exit(2);
    }
    printf("main: global_var == %d\n", *global_var);
    dlclose(handle);
}

void test_dlsym_tls_var(char *lib, int rtld) {
    void *handle = dlopen(lib, rtld);
    if (!handle) {
        printf("dlopen(%s) failed: %s\n", lib, dlerror());
        exit(1);
    }
    int *tls_var = dlsym(handle, "tls_var");
    if (!tls_var) {
        printf("dlsym(handle, tls_var) failed\n");
        exit(2);
    }
    printf("main: tls_var == %d\n", *tls_var);
    dlclose(handle);
}

void test_dladdr_function(char *lib, int rtld) {
    void *handle = dlopen(lib, rtld);
    if (!handle) {
        printf("dlopen(%s) failed: %s\n", lib, dlerror());
        exit(1);
    }

    void (*f)();
    *(void **)(&f) = dlsym(handle, "print");

    if (!f) {
        printf("dlsym(handle, print) failed\n");
        exit(2);
    }

    Dl_info info;
    if (dladdr((void *)(uintptr_t)f, &info) != 0) {
        printf("main: (%s, %s)\n", strrchr(info.dli_fname, '/') + 1, info.dli_sname);
    } else {
        printf("dladdr(print) failed: %s\n", dlerror());
    }

    dlclose(handle);
}

void test_dlunload(char *lib, int rtld) {
    void *handle = dlopen(lib, rtld | RTLD_LOCAL);
    void *handle2 = dlopen(lib, rtld | RTLD_LOCAL);
    assert(handle == handle2 && handle);
    assert(!dlclose(handle));
    void *handle3 = dlopen(lib, rtld | RTLD_GLOBAL);
    assert(handle3);
    assert(!dlclose(handle3));
    if (rtld == RTLD_NOW) return; // unimplemented
    void *handle4 = dlopen(lib, rtld | RTLD_NOLOAD);
    assert(handle4 == handle3);
    assert(!dlclose(handle4));
    assert(!dlclose(handle3));
    void *handle5 = dlopen(lib, rtld | RTLD_NOLOAD);
    assert(handle5 == NULL);
    void *handle6 = dlopen(lib, rtld | RTLD_NOLOAD);
    assert(handle6 == NULL);
}

void test(int rtld) {
    test_dlopen_null(rtld);
    test_dlopen_libc(rtld);
    printf("--sharedlib.so--\n");
    test_dlsym_function(SHARED_LIB, rtld);
    test_dlsym_global_var(SHARED_LIB, rtld);
    test_dlsym_tls_var(SHARED_LIB, rtld);
    test_dladdr_function(SHARED_LIB, rtld);
    test_dlunload(SHARED_LIB, rtld);
    printf("--sharedlib_cpp.so--\n");
    test_dlsym_function(SHARED_LIB_CPP, rtld);
    test_dlsym_global_var(SHARED_LIB_CPP, rtld);
    test_dlsym_tls_var(SHARED_LIB_CPP, rtld);
    test_dladdr_function(SHARED_LIB_CPP, rtld);
    test_dlunload(SHARED_LIB_CPP, rtld);
}

int main() {
    printf("--RTLD_LAZY--\n");
    test(RTLD_LAZY);
    printf("--RTLD_NOW--\n");
    test(RTLD_NOW);
}

