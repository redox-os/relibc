#include <unistd.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    int fd = open("..", 0, 0);
    if (fd == -1) {
        perror("open");
        exit(EXIT_FAILURE);
    } else if (fd < 0) {
        printf("open returned %d, unexpected result\n", fd);
        exit(EXIT_FAILURE);
    }

    int status = fchdir(fd);
    if (status == -1) {
        perror("fchdir");
        exit(EXIT_FAILURE);
    } else if (status != 0) {
        printf("fchdir returned %d, unexpected result\n", status);
        exit(EXIT_FAILURE);
    }

    int c = close(fd);
    if (c == -1) {
        perror("close");
        exit(EXIT_FAILURE);
    } else if (c != 0) {
        printf("close returned %d, unexpected result\n", c);
        exit(EXIT_FAILURE);
    }
}
