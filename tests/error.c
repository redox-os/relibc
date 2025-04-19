#include <assert.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "test_helpers.h"

int main(void) {
    chdir("nonexistent");
    int err = errno;

    printf("errno: %d = %s\n", err, strerror(errno));
    perror("perror");

    char buf1[256] = {0};
    int ret1 = strerror_r(err, buf1, 256);
    printf("errno: %d = %s, return: %d\n", err, buf1, ret1);

    char buf2[3] = {0};
    int ret2 = strerror_r(err, buf2, 3);
    printf("errno: %d = %s, return: %d\n", err, buf2, ret2);

    char buf3[256] = {0};
    int ret3 = strerror_r(err, buf3, 0);
    printf("errno: %d = %s, return: %d\n", err, buf3, ret3);

    // Test that the statically allocated buffer doesn't overflow
    for (int code = EPERM; code < ENOTRECOVERABLE; ++code) {
        const char* message = strerror(code);
        assert(message);
    }

    return EXIT_SUCCESS;
}
