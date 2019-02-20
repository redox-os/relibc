#include <unistd.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    int fd = open(".", 0, 0);
    int status;
    status = fsync(fd);
    printf("fsync exited with status code %d\n", status);
    close(fd);
    return EXIT_SUCCESS;
}
