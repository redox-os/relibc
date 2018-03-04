#include <fcntl.h>
#include <unistd.h>

int main(int argc, char **argv) {
    int fd = creat("create.out", 0755);
    if (fd >= 0) {
        write(fd, "Hello World!\n", 13);
        close(fd);
        return 0;
    } else {
        write(STDERR_FILENO, "creat failed\n", 13);
        return 1;
    }
}
