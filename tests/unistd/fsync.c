#include <unistd.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    int fd = open("example_dir/1-never-gonna-give-you-up", O_RDWR);
    ERROR_IF(open, fd, == -1);
    UNEXP_IF(open, fd, < 0);

    int status = fsync(fd);
    ERROR_IF(fsync, status, == -1);
    UNEXP_IF(fsync, status, != 0);

    int c = close(fd);
    ERROR_IF(close, c, == -1);
    UNEXP_IF(close, c, != 0);
}
