#include <stdio.h>
#include <unistd.h>

#include "test_helpers.h"

int main(void) {
    const char *msg = "Hello, %s";

    int fd = dup(STDOUT_FILENO);
    ERROR_IF(dup, fd, == -1);

    int result = dprintf(fd, msg, "world");
    ERROR_IF(dprintf, result, != sizeof("Hello, world") - 1);
    UNEXP_IF(dprintf, result, < 0);

    result = dprintf(fd, "\na\n");
    UNEXP_IF(dprintf, result, < 0);
    ERROR_IF(dprintf, result, != sizeof("\na\n") - 1);

    close(fd);

    return 0;
}
