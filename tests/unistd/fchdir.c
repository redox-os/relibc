#include <unistd.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    int fd = open("..", O_DIRECTORY);
    ERROR_IF(open, fd, == -1);
    UNEXP_IF(open, fd, < 0);

    int status = fchdir(fd);
    ERROR_IF(fchdir, status, == -1);
    UNEXP_IF(fchdir, status, != 0);

    int c = close(fd);
    ERROR_IF(close, c, == -1);
    UNEXP_IF(close, c, != 0);
}
