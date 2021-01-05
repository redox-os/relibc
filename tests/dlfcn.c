#include <stdio.h>
#include <stdlib.h>
#include <dlfcn.h>


int add(int a, int b)
{
    return a + b;
}

void test_dlopen_null()
{
    void* handle = dlopen(NULL, RTLD_LAZY);
    if (!handle) {
        printf("dlopen(NULL) failed\n");
        exit(1);
    }
    int (*f)(int, int) = dlsym(handle, "add");
    if (!f) {
        printf("dlsym(handle, add) failed\n");
        exit(2);
    }
    int a = 22;
    int b = 33;
    printf("add(%d, %d) = %d\n", a, b, f(a, b));
    dlclose(handle);
}

void test_dlopen_libc()
{
    void* handle = dlopen("libc.so.6", RTLD_LAZY);
    if (!handle) {
        printf("dlopen(libc.so.6) failed\n");
        exit(1);
    }
    int (*f)(const char*) = dlsym(handle, "puts");
    if (!f) {
        printf("dlsym(handle, puts) failed\n");
        exit(2);
    }
    f("puts from dlopened libc");
    dlclose(handle);
}


void test_dlsym_function()
{
    void* handle = dlopen("sharedlib.so", RTLD_LAZY);
    if (!handle) {
        printf("dlopen(sharedlib.so) failed\n");
        exit(1);
    }
    void (*f)() = dlsym(handle, "print");
    if (!f) {
        printf("dlsym(handle, print) failed\n");
        exit(2);
    }
    f();
    dlclose(handle);
}

void test_dlsym_global_var()
{
    void* handle = dlopen("sharedlib.so", RTLD_LAZY);
    if (!handle) {
        printf("dlopen(sharedlib.so) failed\n");
        exit(1);
    }
    int* global_var = dlsym(handle, "global_var");
    if (!global_var) {
        printf("dlsym(handle, global_var) failed\n");
        exit(2);
    }
    printf("main: global_var == %d\n", *global_var);
    dlclose(handle);
}

void test_dlsym_tls_var()
{
    void* handle = dlopen("sharedlib.so", RTLD_LAZY);
    if (!handle) {
        printf("dlopen(sharedlib.so) failed\n");
        exit(1);
    }
    int* tls_var = dlsym(handle, "tls_var");
    if (!tls_var) {
        printf("dlsym(handle, tls_var) failed\n");
        exit(2);
    }
    printf("main: tls_var == %d\n", *tls_var);
    dlclose(handle);
}

int main()
{
    test_dlopen_null();
    test_dlopen_libc();
    test_dlsym_function();
    test_dlsym_global_var();
    test_dlsym_tls_var();
}

