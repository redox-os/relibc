#ifndef _TEST_HELPERS
#define _TEST_HELPERS

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

// Throws an error on a well-defined error value.
// Don't pass functions as status or condition, it might evaluate them multiple times.
#define ERROR_IF(func, status, condition) { \
    if (status condition) { \
        fprintf(stderr, "%s:%s:%d: '%s' returned an error: %s (%d)\n", \
            __FILE__, __func__, __LINE__, #func, strerror(errno), errno); \
        _exit(EXIT_FAILURE); \
    }\
}

// Throws an error on an return value not defined by the standards.
// Used for sanity checking the return values.
// Don't pass functions as status or condition it might evaluate them multiple times.
#define UNEXP_IF(func, status, condition) { \
    if (status condition) { \
        fprintf(stderr, "%s:%s:%d: '%s' returned a non-standard value: %d\n", \
            __FILE__, __func__, __LINE__, #func, status); \
        _exit(EXIT_FAILURE); \
    }\
}

// A convenience macro to show where the test fail.
#define exit(code) { \
    if (code != EXIT_SUCCESS) { \
        fprintf(stderr, "%s:%s:%d: Test failed with exit(%s)\n", \
            __FILE__, __func__, __LINE__, #code); \
    } \
    _exit(code); \
}

#endif /* _TEST_HELPERS */
