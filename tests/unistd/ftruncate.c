#include <unistd.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    int fd = creat("ftruncate.out", 0777);
    ERROR_IF(creat, fd, == -1);
    UNEXP_IF(creat, fd, < 0);

    int status = ftruncate(fd, 100);
    ERROR_IF(ftruncate, status, == -1);
    UNEXP_IF(ftruncate, status, != 0);

    int c = close(fd);
    ERROR_IF(close, c, == -1);
    UNEXP_IF(close, c, != 0);

    status = truncate("ftruncate.out", 100);
    ERROR_IF(truncate, status, == -1);
    UNEXP_IF(truncate, status, != 0);
}
