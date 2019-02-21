#include <fcntl.h>
#include <stdlib.h>
#include <unistd.h>

#include "test_helpers.h"

int main(void) {
    int fd = creat("create.out", 0755);
    ERROR_IF(creat, fd, == -1);
    UNEXP_IF(creat, fd, < 0);

    write(fd, "Hello World!\n", 13);
    close(fd);
}
