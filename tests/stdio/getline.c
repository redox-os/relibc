// https://pubs.opengroup.org/onlinepubs/9699919799/functions/getline.html

/// ssize_t getdelim(char **restrict lineptr, size_t *restrict n,
///                  int delimiter, FILE *restrict stream);
/// ssize_t getline(char **restrict lineptr, size_t *restrict n,
///                 FILE *restrict stream);

#include <assert.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

const char   *INFILE_NAME   = "stdio/getline.in";
const size_t INFILE_LINES   = 7;
const size_t OVERREAD_LINES = 3;

void test_null_args();

void test_ferror();

void test_read_and_overread();

int main(void) {
    setbuf(stdout, NULL);

    // test if supplying NULL for either pointer correctly yields EINVAL
    test_null_args();

    // test reading stream with error flag set
    test_ferror();

    // "normal" case - read all `INFILE_LINES` lines of the test file and then overread by 3
    test_read_and_overread();
}

void test_null_args() {


    // Basics: NULL handling
    // Test all combinations of NULL pointers since we fail on any one.
    //
    // I can't explicitly find that stream can't be NULL but nothing else
    // makes sense, so let's return EINVAL
#define TEST_FOR_EINVAL(x)                           \
    do                                               \
    {                                                \
        size_t n = 0;                                \
        char *lineptr = NULL;                        \
        ssize_t status = 0;                          \
        FILE *stream = NULL;                         \
                                                     \
        stream = fopen(INFILE_NAME, "r");            \
        ERROR_IF(fopen, stream, == NULL);            \
        ERROR_IF(fopen, ferror(stream), );           \
                                                     \
        status = x;                                  \
        printf("%3zu: %s\n     => ", ++counter, #x); \
        CHECK_AND_PRINT_ERRNO(EINVAL);               \
        assert(status == -1 && errno == EINVAL);     \
                                                     \
        if (stream != NULL)                          \
            fclose(stream);                          \
        if (lineptr != NULL)                         \
            free(lineptr);                           \
        stream = NULL;                               \
        errno = 0;                                   \
        (void)n;                                     \
    } while (0);

        static size_t counter = 0;

        TEST_FOR_EINVAL(getline(NULL, NULL, stream));

        TEST_FOR_EINVAL(getline(NULL, &n, stream));
        TEST_FOR_EINVAL(getline(&lineptr, NULL, stream));

        // don't always use, as glibc doesn't tolerate stream being NULL or delim being out-of-range for char
#ifdef _RELIBC_STDIO_H
        TEST_FOR_EINVAL(getline(NULL, NULL, NULL));
        TEST_FOR_EINVAL(getline(&lineptr, NULL, NULL));
        TEST_FOR_EINVAL(getline(NULL, &n, NULL));
        TEST_FOR_EINVAL(getline(&lineptr, &n, NULL));

        // test if using delim out of char range correctly causes EINVAL
        // POSIX specifies UB, so we try to be more helpful
        TEST_FOR_EINVAL(getdelim(&lineptr, &n, 25600, stream));
#endif

#undef TEST_FOR_EINVAL
    printf("\n");
}

void test_ferror() {
    // TODO test behavior on error flag set
}

void test_read_and_overread() {
    // Basic use cases
    // Read all INFILE_LINES lines of sample input, then overread OVERREAD_LINES times

    size_t  n        = 0;
    char    *lineptr = NULL;
    ssize_t status;
    FILE    *stream  = NULL;

    stream = fopen(INFILE_NAME, "r");

    for (size_t i = 0; i < INFILE_LINES; ++i) {
        /// "Upon successful completion, the getline() and getdelim() functions
        /// shall return the number of bytes written into the buffer, including
        /// the delimiter character if one was encountered before EOF, but
        /// excluding the terminating NUL character."
        printf("%3zu: ", i + 1);
        status = getline(&lineptr, &n, stream);

        /// "If the end-of-file indicator for the stream is set, or if no
        /// characters were read and the stream is at end-of-file, the
        /// end-of-file indicator for the stream shall be set and the function
        /// shall return -1."
        assert(!(status == -1 && feof(stream)));

        /// "If an error occurs, the error indicator for the stream shall be
        /// set, and the function shall return -1 and set errno to indicate the
        /// error."
        ERROR_IF(getline, status, == -1 && ferror(stream));

        // This should only execute if the other error cases (with stream flags set) did not abort.
        UNEXP_IF(getline, status, == -1);

        // Print length and content to verify. Also test if the buffer is big
        // enough and if strlen of input matches the return value.
        // Although this doesn't HAVE to be true, as the input could contain
        // NUL bytes, ours doesn't.
        printf("status = %zi, strlen = %zu, feof = %s, ferror = %s\n     |>%s",
               status, strlen(lineptr), feof(stream) ? "1" : "0", ferror(stream) ? "1" : "0", lineptr);
        // fflush(stdout);

        assert(strlen(lineptr) == (size_t) status); // we can cast to size_t since we
        assert(n >= (size_t) status + 1);           // UNEXP_IF against status being -1
    }
    printf("\n");

    // OVERREAD

    for (size_t i = 0; i < OVERREAD_LINES; ++i) {
        /// "If the end-of-file indicator for the stream is set, or if no
        /// characters were read and the stream is at end-of-file, the
        /// end-of-file indicator for the stream shall be set and the function
        /// shall return -1."
        status = getline(&lineptr, &n, stream);
        printf("overread %zu, status = %zi, feof = %s, ferror = %s\n",
                i + 1,
                status,
                feof(stream) ? "1" : "0", ferror(stream) ? "1" : "0");
        if (i == 0) {
            assert(status == -1);
            assert(feof(stream));
            assert(!ferror(stream));
        }
        printf("|~%s\n", lineptr);
    }

    // cleanup
    fclose(stream);
    free(lineptr);
}
