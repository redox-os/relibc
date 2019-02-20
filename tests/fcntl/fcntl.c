#include <fcntl.h>
#include <stdio.h>
#include <unistd.h>

int main(void) {
    //Lose our fd and pull it again
    creat("fcntl.out", 0777);
    int newfd = open("fcntl.out", 0);
    int newfd2 = fcntl(newfd, F_DUPFD, 0);
    printf("fd %d duped into fd %d\n", newfd, newfd2);
    close(newfd);
    close(newfd2);
}
