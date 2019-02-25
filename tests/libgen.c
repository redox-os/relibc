#include <libgen.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

#include "test_helpers.h"

#define TODO NULL

// TODO: Tests for Redox schemes
struct test_case {
    char *path;
    char *dirname;
    char *basename;
} test_cases[] = {
    // Classic UNIX
    // path              dirname  basename
    { "",                ".",     "."   },
    { ".",               ".",     "."   },
    { "..",              ".",     ".."  },
    { "/",               "/",     "/"   },
    { "///",             "/",     "/"   },
    { "//usr//lib//",    "//usr", "lib" },
    { "/usr",            "/",     "usr" },
    { "/usr/",           "/",     "usr" },
    { "/usr/lib",        "/usr",  "lib" },
    { NULL,              ".",     "."   }
    // Root scheme
    // path              dirname  basename
    //{ ":",               TODO,    TODO },
    //{ ":/",              TODO,    TODO },
    //{ ":/scheme",        TODO,    TODO },
    // Regular scheme
    // path              dirname  basename
    //{ "file:",           TODO,    TODO },
    //{ "file:usr",        TODO,    TODO },
    //{ "file:usr/",       TODO,    TODO },
    //{ "file:usr/lib",    TODO,    TODO },
    //{ "file:/",          TODO,    TODO },
    //{ "file:/usr",       TODO,    TODO },
    //{ "file:/usr/",      TODO,    TODO },
    //{ "file:/usr/lib",   TODO,    TODO },
    //{ "file:///",        TODO,    TODO },
    //{ "file://usr",      TODO,    TODO },
    //{ "file://usr//",    TODO,    TODO },
    // Hierarchical scheme
    // path              dirname  basename
    //{ "disk/0:",         TODO,    TODO },
    //{ "disk/0:/",        TODO,    TODO },
    //{ "disk/0:///",      TODO,    TODO },
    //{ "disk/0:/usr",     TODO,    TODO },
    //{ "disk/0:/usr/",    TODO,    TODO },
    //{ "disk/0:/usr/lib", TODO,    TODO },
    // Malformed
    // path              dirname  basename
    //{ "/file:/sys:/usr", TODO,    TODO },
    //{ "/file:/usr",      TODO,    TODO },
    //{ "/file:sys:/usr",  TODO,    TODO },
    //{ "/file:usr",       TODO,    TODO },
    //{ ":file/usr",       TODO,    TODO },
    //{ "file:/sys:/usr",  TODO,    TODO },
    //{ "file::/",         TODO,    TODO },
    //{ "file::/usr/lib",  TODO,    TODO }
};

size_t num_test_cases = sizeof(test_cases) / sizeof(struct test_case);

int safe_strcmp(char *s1, char *s2) {
    if (s1 == NULL && s2 == NULL) {
        return 0;
    } else if (s1 == NULL && s2 != NULL) {
        return 1;
    } else if (s1 != NULL && s2 == NULL) {
        return -1;
    } else {
        return strcmp(s1, s2);
    }
}

#define CHECK_TEST(tc, fn, retval)                                             \
    do {                                                                       \
        /* API for basename and dirname allow the passed in string to */       \
        /* be modified. This means we have to pass a modifiable copy. */       \
        char *path = NULL;                                                     \
        if (tc.path != NULL)                                                   \
            path = strdup(tc.path);                                            \
                                                                               \
        char *output = fn(path);                                               \
                                                                               \
        /* Printing NULLs with printf("%s") is undefined behaviour, */         \
        /* that's why they are handled here this way.               */         \
        char display_path[64] = "NULL";                                        \
        char display_output[64] = "NULL";                                      \
        char display_expected[64] = "NULL";                                    \
        if (tc.path != NULL) sprintf(display_path, "\"%s\"", tc.path);         \
        if (output != NULL) sprintf(display_output, "\"%s\"", output);         \
        if (tc.fn != NULL) sprintf(display_expected, "\"%s\"", tc.fn);         \
                                                                               \
        if (safe_strcmp(output, tc.fn) != 0) {                                 \
            retval = EXIT_FAILURE;                                             \
            printf("%s(%s) != %s, expected: %s\n",                             \
                #fn, display_path, display_output, display_expected);          \
        } else {                                                               \
            printf("%s(%s) == %s\n",                                           \
                #fn, display_path, display_output);                            \
        }                                                                      \
                                                                               \
        free(path);                                                            \
    } while (0)

int main(void) {
    int retval = EXIT_SUCCESS;

    for(int i = 0; i < num_test_cases; ++i) {
        struct test_case tc = test_cases[i];
        CHECK_TEST(tc, dirname, retval);
        CHECK_TEST(tc, basename, retval);
    }

    if (retval == EXIT_SUCCESS) {
        printf("Success: %d\n", retval);
    } else {
        printf("Failure: %d\n", retval);
    }

    return retval;
}
