#include <errno.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#include "test_helpers.h"

int main(void) {
    int sv[2];
    pid_t pid;
    char buf[64];
    const char *parent_msg = "ping";
    const char *child_msg = "pong";

    int status = socketpair(AF_UNIX, SOCK_STREAM, 0, sv);
    ERROR_IF(socketpair, status, == -1);

    pid = fork();
    ERROR_IF(fork, pid, == -1);

    if (pid == 0) {
        close(sv[0]);

        memset(buf, 0, sizeof(buf));
        ssize_t n = recv(sv[1], buf, sizeof(buf), 0);
        ERROR_IF(recv, n, == -1);
        printf("child:  received '%s'\n", buf);

        n = recv(sv[1], buf, sizeof(buf), 0);
        if (n != 0) {
            fprintf(
                stderr,
                "FAILURE: Expected EOF (0) after parent shutdown, got %ld\n",
                (long)n);
            exit(EXIT_FAILURE);
        }
        printf("child:  verified EOF from parent (SHUT_WR worked)\n");

        status = send(sv[1], child_msg, 5, 0);
        ERROR_IF(send, status, == -1);

        close(sv[1]);
        exit(0);
    } else {
        close(sv[1]);

        status = send(sv[0], parent_msg, 5, 0);
        ERROR_IF(send, status, == -1);

        status = shutdown(sv[0], SHUT_WR);
        ERROR_IF(shutdown, status, == -1);
        printf("parent: shutdown(SHUT_WR) performed\n");

        memset(buf, 0, sizeof(buf));
        ssize_t n = recv(sv[0], buf, sizeof(buf), 0);
        ERROR_IF(recv, n, == -1);

        if (n == 0) {
            fprintf(stderr,
                    "FAILURE: Parent received EOF, but expected 'pong'\n");
            exit(EXIT_FAILURE);
        }
        printf("parent: received '%s' (Shutdown allows reading)\n", buf);

        status = send(sv[0], "garbage", 7, MSG_NOSIGNAL);
        if (status != -1 || errno != EPIPE) {
            fprintf(stderr,
                    "WARNING: Expected EPIPE on write after shutdown, got "
                    "status %d\n",
                    status);
        } else {
            printf("parent: verified EPIPE on write after shutdown\n");
        }

        close(sv[0]);
        wait(NULL);
    }

    return 0;
}
