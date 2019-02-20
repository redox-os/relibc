#include <fcntl.h>
#include <stdio.h>
#include <sys/select.h>
#include <unistd.h>

int main(void) {
    int fd = open("select.c", 0, 0);

    fd_set read;
    FD_ZERO(&read);
    FD_SET(fd, &read);

    printf("Is set before? %d\n", FD_ISSET(fd, &read));

    // This should actually test TCP streams and stuff, but for now I'm simply
    // testing whether it ever returns or not.
    printf("Amount of things ready: %d\n", select(fd + 1, &read, NULL, NULL, NULL));

    printf("Is set after? %d\n", FD_ISSET(fd, &read));

    close(fd);
}
