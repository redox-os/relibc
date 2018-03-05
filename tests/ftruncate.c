#include <unistd.h>
#include <fcntl.h>
#include <stdio.h>

int main (int argc, char** argv) {
    int fd = creat("ftruncate.out", 0777); 
    int status;
    status = ftruncate(fd, 100);
    printf("ftruncate exited with status code %d\n", status);
    close(fd);
}
