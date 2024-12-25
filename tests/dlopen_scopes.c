#include <assert.h>
#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    void *handle = dlopen("libfoobar.so", RTLD_LAZY | RTLD_LOCAL);
    if (!handle) {
        printf("dlopen(libfoobar.so): %s\n", dlerror());
        return EXIT_FAILURE;
    }

    assert(dlsym(handle, "BAR") != NULL);
    assert(dlsym(handle, "FOO") != NULL);
    // not in the global scope
    assert(dlsym(RTLD_DEFAULT, "BAR") == NULL);
    assert(dlsym(RTLD_DEFAULT, "FOO") == NULL);

    void *self = dlopen(NULL, RTLD_LAZY | RTLD_LOCAL);
    if (!self) {
        printf("dlopen(NULL): %s\n", dlerror());
        return EXIT_FAILURE;
    }

    assert(dlsym(self, "FOO") == NULL);
    assert(dlsym(self, "BAR") == NULL);

    assert(dlclose(self) == 0);

    // Promote the library to the global scope.
    assert(dlopen("libfoobar.so", /* RTLD_NOLOAD |*/ RTLD_NOW | RTLD_GLOBAL));

    assert(dlsym(RTLD_DEFAULT, "FOO") != NULL);
    assert(dlsym(RTLD_DEFAULT, "BAR") != NULL);

    return EXIT_SUCCESS;
}
