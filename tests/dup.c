#include <unistd.h>
#include <fcntl.h>
#include <stdio.h>

int main(int argc, char** argv) {
    creat("dup.out", 0777);
    int fd1 = open("dup.out", 0, 0);
    int fd2 = dup(fd1);
    printf("fd %d duped into fd %d\n", fd1, fd2);
    close(fd1);
    close(fd2);
    int fd3 = open("dup.out", 0x0002, 0x1000);
    dup2(fd3, 1);
    printf("hello fd %d", fd3);
    close(fd3);
}
