#include <fcntl.h>
#include <stdio.h>
#include <unistd.h>

#include "test_helpers.h"

int main(void) {
    // Lose our fd and pull it again
    {
        int fd = creat("fcntl.out", 0777);
        ERROR_IF(creat, fd, == -1);
        UNEXP_IF(creat, fd, < 0);
    }

    int newfd = open("fcntl.out", 0);
    ERROR_IF(open, newfd, == -1);
    UNEXP_IF(open, newfd, < 0);

    int newfd2 = fcntl(newfd, F_DUPFD, 0);
    // TODO: The standard doesn't define errors for F_DUPFD

    printf("duped fd is %d greater than the original fd\n", newfd2 - newfd);

    int c1 = close(newfd);
    ERROR_IF(close, c1, == -1);
    UNEXP_IF(close, c1, != 0);

    int c2 = close(newfd2);
    ERROR_IF(close, c2, == -1);
    UNEXP_IF(close, c2, != 0);
}
