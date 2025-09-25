#define _GNU_SOURCE

#include <fcntl.h>
#include <stdio.h>
#include <sys/select.h>
#include <unistd.h>

const struct timeval TIMEOUT = {
    .tv_sec = 5,
    .tv_usec = 0
};

static int file_test(void) {
    int fd = open("select.c", 0, 0);
    if (fd < 0) {
        perror("open");
        return -1;
    }

    printf("Testing select on file\n");

    fd_set read;
    FD_ZERO(&read);
    FD_SET(fd, &read);

    printf("Is set before? %d\n", FD_ISSET(fd, &read));

    struct timeval timeout = TIMEOUT;
    int nfds = select(fd + 1, &read, NULL, NULL, &timeout);
    if (timeout.tv_sec == 0) {
        fputs("Timeout hit for file test\n", stderr);
        return 1;
    }
    if (nfds < 0) {
        perror("select");
        return 1;
    }
    printf("Amount of things ready: %d\n", nfds);

    printf("Is set after? %d\n", FD_ISSET(fd, &read));

    close(fd);

    return 0;
}

static int pipe_test(void) {
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

    printf("Testing select on pipe\n");

    fd_set read;
    FD_ZERO(&read);
    FD_SET(pipefd[0], &read);

    printf("Is set before? %d\n", FD_ISSET(pipefd[0], &read));

    struct timeval timeout = TIMEOUT;
    int nfds = select(pipefd[0] + 1, &read, NULL, NULL, &timeout);
    if (timeout.tv_sec == 0) {
        fputs("Timeout hit for pipe test\n", stderr);
        return 1;
    }
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

int main(void) {
    if (file_test()) {
        return 1;
    }

    if (pipe_test()) {
        return 1;
    }

    return 0;
}
