#include <unistd.h>
#include <fcntl.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    int fd0 = creat("dup.out", 0777);
    ERROR_IF(creat, fd0, == -1);
    UNEXP_IF(creat, fd0, < 0);

    int fd1 = open("dup.out", 0);
    ERROR_IF(open, fd1, == -1);
    UNEXP_IF(open, fd1, < 0);

    int fd2 = dup(fd1);
    ERROR_IF(dup, fd2, == -1);
    UNEXP_IF(dup, fd2, < 0);

    printf("duped fd is %d greater than the original fd\n", fd2 - fd1);

    int c1 = close(fd1);
    ERROR_IF(close, c1, == -1);
    UNEXP_IF(close, c1, != 0);

    int c2 = close(fd2);
    ERROR_IF(close, c2, == -1);
    UNEXP_IF(close, c2, != 0);

    int fd3 = open("dup.out", 0x0002, 0x1000);
    ERROR_IF(open, fd3, == -1);
    UNEXP_IF(open, fd3, < 0);

    int fd4 = dup2(fd3, 1);
    ERROR_IF(dup2, fd4, == -1);
    UNEXP_IF(dup2, fd4, < 0);

    printf("hello fd %d", fd3);

    int c3 = close(fd3);
    ERROR_IF(close, c3, == -1);
    UNEXP_IF(close, c3, != 0);
}
