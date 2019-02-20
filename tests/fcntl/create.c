#include <fcntl.h>
#include <stdlib.h>
#include <unistd.h>

int main(void) {
    int fd = creat("create.out", 0755);
    if (fd >= 0) {
        write(fd, "Hello World!\n", 13);
        close(fd);
        return EXIT_SUCCESS;
    } else {
        write(STDERR_FILENO, "creat failed\n", 13);
        return EXIT_FAILURE;
    }
}
