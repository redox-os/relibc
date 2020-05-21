#include <unistd.h>
#include <stdio.h>
#include <string.h>
#include <errno.h>

#include "test_helpers.h"

int main(void) {
    chdir("nonexistent");
    int err = errno;

    printf("errno: %d = %s\n", err, strerror(errno));
    perror("perror");

    char buf1[256];
    int ret1 = strerror_r(err, buf1, 256);
    printf("errno: %d = %s, return: %d\n", err, buf1, ret1);

    char buf2[3];
    int ret2 = strerror_r(err, buf2, 3);
    printf("errno: %d = %s, return: %d\n", err, buf2, ret2);

    char buf3[256] = {0};
    int ret3 = strerror_r(err, buf3, 0);
    printf("errno: %d = %s, return: %d\n", err, buf3, ret3);
}
