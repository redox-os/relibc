#include <unistd.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    int fd = creat("ftruncate.out", 0777);
    if (fd == -1) {
        perror("creat");
        exit(EXIT_FAILURE);
    } else if (fd < 0) {
        printf("creat returned %d, unexpected result\n", fd);
        exit(EXIT_FAILURE);
    }

    int status = ftruncate(fd, 100);
    if (status == -1) {
        perror("ftruncate");
        exit(EXIT_FAILURE);
    } else if (status != 0) {
        printf("ftruncate returned %d, unexpected result\n", status);
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
