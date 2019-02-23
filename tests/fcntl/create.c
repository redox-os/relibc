#include <fcntl.h>
#include <stdlib.h>
#include <unistd.h>

#include "test_helpers.h"

int main(void) {
    int fd = creat("create.out", 0755);
    ERROR_IF(creat, fd, == -1);
    UNEXP_IF(creat, fd, < 0);

    int written = write(fd, "Hello World!\n", 13);
    ERROR_IF(write, written, == -1);

    int c = close(fd);
    ERROR_IF(close, c, == -1);
    UNEXP_IF(close, c, != 0);
}
