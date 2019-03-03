#ifndef _TEST_HELPERS
#define _TEST_HELPERS

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

// Throws errors on a well-defined API error values.
//
// Only use with API functions that sets the errno variable.
// Do not pass functions as the status or condition arguments, they might be
// evaluated multiple times.
//
// Usage example:
//
// > Upon successful completion, fclose() returns 0.
// > Otherwise, it returns EOF and sets errno to indicate the error.
//
// int status = fclose(fp);
// ERROR_IF(fclose, status, == EOF);
//
// Use it only for checking the API error values.
// Do not use it for checking the correctness of the results. If you need to
// do that, print the values to the standard output and use the expected outputs
// directory.
//
// For example:
//
// int c = fgetc(f);            // !!! DO NOT USE THIS WAY !!!
// ERROR_IF(fgetc, c, != 'H');  // !!! DO NOT USE THIS WAY !!!
//
// Correct usage:
//
// int c = fgetc(f);            // OK
// ERROR_IF(fgetc, c, == EOF);  // OK
// printf("result: %c\n", c);   // OK
//
#define ERROR_IF(func, status, condition) \
    do { \
        if (status condition) { \
            fprintf(stderr, "%s:%s:%d: '%s' failed: %s (%d)\n", \
                __FILE__, __func__, __LINE__, #func, strerror(errno), errno); \
            _exit(EXIT_FAILURE); \
        } \
    } while(0)

// Throws errors on API return values not defined by the standards.
//
// Do not pass functions as the status or condition arguments, they might be
// evaluated multiple times.
//
// Use it only for detecting return values that should have never been returned
// in any case by the API functions.
//
// Usage example:
//
// > The fgetc() function obtains the next byte as an unsigned char
// > converted to an int.
//
// int c = fgetc(f);
// UNEXP_IF(fgetc, c, < 0);
// UNEXP_IF(fgetc, c, > 255);
//
#define UNEXP_IF(func, status, condition) \
    do { \
        if (status condition) { \
            fprintf(stderr, "%s:%s:%d: '%s' returned a non-standard value: ", \
                __FILE__, __func__, __LINE__, #func); \
            fprintf(stderr, _Generic((status), \
                char *: "char*(%p) = \"%1$s\"", \
                void *: "void*(%p)", \
                default: "%i" \
            ), status); \
            fprintf(stderr, "\n"); \
            _exit(EXIT_FAILURE); \
        } \
    } while (0)

// A convenience macro to show where the test fail.
#define exit(code) \
    do { \
        if (code != EXIT_SUCCESS) { \
            fprintf(stderr, "%s:%s:%d: Test failed with exit(%s)\n", \
                __FILE__, __func__, __LINE__, #code); \
        } \
        _exit(code); \
    } while(0)

#endif /* _TEST_HELPERS */
