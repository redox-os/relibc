#include <fcntl.h>
#include <stdio.h>
#include <sys/select.h>
#include <unistd.h>

#include "test_helpers.h"

int main(void) {
    int pipefd[2];
    if (pipe2(pipefd, O_NONBLOCK) < 0) {
        perror("pipe");
        return 1;
    }

    char c = 'c';
    if (write(pipefd[1], &c, sizeof(c)) < 0) {
        perror("write");
        return 1;
    }

    fd_set read;
    FD_ZERO(&read);
    FD_SET(pipefd[0], &read);

    printf("Is set before? %d\n", FD_ISSET(pipefd[0], &read));

    // This should actually test TCP streams and stuff, but for now I'm simply
    // testing whether it ever returns or not.
    int nfds = select(pipefd[0] + 1, &read, NULL, NULL, NULL);
    if (nfds < 0) {
        perror("select");
        return 1;
    }
    printf("Amount of things ready: %d\n", nfds);

    printf("Is set after? %d\n", FD_ISSET(pipefd[0], &read));

    close(pipefd[0]);
    close(pipefd[1]);

    return 0;
}
