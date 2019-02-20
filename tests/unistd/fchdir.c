#include <unistd.h>
#include <fcntl.h>
#include <stdio.h>

int main(void) {
    int fd = open("..", 0, 0);
    int status;
    status = fchdir(fd);
    printf("fchdir exited with status code %d\n", status);
    close(fd);
}
