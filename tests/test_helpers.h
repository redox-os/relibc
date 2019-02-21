#ifndef _TEST_HELPERS
#define _TEST_HELPERS

#include <stdlib.h>
#include <string.h>
#include <errno.h>

// Throws an error on a well-defined error value.
#define ERROR_IF(func, status, condition) { \
    if (status condition) { \
        fprintf(stderr, "%s:%d: ‘%s‘ returned an error in function ‘%s’: %s (%d)\n", \
            __FILE__, __LINE__, #func, __func__, strerror(errno), errno); \
        exit(EXIT_FAILURE); \
    }\
}

// Throws an error on an return value not defined by the standards.
// Used for sanity checking the return values.
#define UNEXP_IF(func, status, condition) { \
    if (status condition) { \
        fprintf(stderr, "%s:%d: ‘%s‘ returned a value not defined by the standards in function ‘%s’: %d\n", \
            __FILE__, __LINE__, #func, __func__, status); \
        exit(EXIT_FAILURE); \
    }\
}

#endif /* _TEST_HELPERS */
