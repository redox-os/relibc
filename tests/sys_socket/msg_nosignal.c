#include <errno.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>

#include "test_helpers.h"

static volatile sig_atomic_t sigpipe_count = 0;

static void handle_sigpipe(int signal) {
    (void)signal;
    sigpipe_count++;
}

static void test_broken_sendto(int flags, sig_atomic_t expected_sigpipes) {
    int sockets[2];
    int status = socketpair(AF_UNIX, SOCK_STREAM, 0, sockets);
    ERROR_IF(socketpair, status, == -1);

    status = close(sockets[1]);
    ERROR_IF(close, status, == -1);

    sig_atomic_t count_before = sigpipe_count;
    errno = 0;
    ssize_t sent = sendto(sockets[0], "x", 1, flags, NULL, 0);
    int send_errno = errno;

    if (sent != -1 || send_errno != EPIPE) {
        fprintf(stderr,
                "sendto(flags=%#x): expected -1/EPIPE, got %ld/%d (%s)\n",
                flags,
                (long)sent,
                send_errno,
                strerror(send_errno));
        exit(EXIT_FAILURE);
    }

    sig_atomic_t delivered = sigpipe_count - count_before;
    if (delivered != expected_sigpipes) {
        fprintf(stderr,
                "sendto(flags=%#x): expected %d SIGPIPE signal(s), got %d\n",
                flags,
                (int)expected_sigpipes,
                (int)delivered);
        exit(EXIT_FAILURE);
    }

    status = close(sockets[0]);
    ERROR_IF(close, status, == -1);
}

int main(void) {
    struct sigaction action = {0};
    action.sa_handler = handle_sigpipe;
    action.sa_flags = 0;

    int status = sigemptyset(&action.sa_mask);
    ERROR_IF(sigemptyset, status, == -1);

    status = sigaction(SIGPIPE, &action, NULL);
    ERROR_IF(sigaction, status, == -1);

    test_broken_sendto(MSG_NOSIGNAL, 0);
    test_broken_sendto(0, 1);

    return EXIT_SUCCESS;
}
