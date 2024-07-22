#include <stdio.h>
#include <fcntl.h>

#include "test_helpers.h"

int main(void)
{
    int fd = open("/dev/stdout", O_WRONLY, 0222);
    ERROR_IF(open, fd, < 0);

    const char *msg = "Hello, %s";

    int result = dprintf(fd, msg, "world");
    ERROR_IF(dprintf, result, != sizeof("Hello, world") - 1);
    UNEXP_IF(dprintf, result, < 0);

    result = dprintf(fd, "\na\n");
    UNEXP_IF(dprintf, result, < 0);
    ERROR_IF(dprintf, result, != sizeof("\na\n") - 1);

    close(fd);

    return 0;
}