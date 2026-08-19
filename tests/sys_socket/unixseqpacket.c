/* SOCK_SEQPACKET preserves message boundaries: each recv() returns exactly one
 * message, and a message longer than the supplied buffer is truncated rather
 * than continued by the following recv(). */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>

#include "test_helpers.h"

int main(void) {
    int sv[2];
    char buf[64];

    int status = socketpair(AF_UNIX, SOCK_SEQPACKET, 0, sv);
    ERROR_IF(socketpair, status, == -1);

    /* Two separate messages must not be coalesced into one recv(). */
    ssize_t sent = send(sv[0], "abc", 3, 0);
    ERROR_IF(send, sent, == -1);
    sent = send(sv[0], "defgh", 5, 0);
    ERROR_IF(send, sent, == -1);

    ssize_t n = recv(sv[1], buf, sizeof(buf), 0);
    ERROR_IF(recv, n, == -1);
    printf("first:  %zd %.*s\n", n, (int)n, buf);

    n = recv(sv[1], buf, sizeof(buf), 0);
    ERROR_IF(recv, n, == -1);
    printf("second: %zd %.*s\n", n, (int)n, buf);

    /* A message that does not fit is truncated, and the remainder is dropped
     * instead of being returned by the next recv(). */
    sent = send(sv[0], "ijklm", 5, 0);
    ERROR_IF(send, sent, == -1);
    sent = send(sv[0], "nop", 3, 0);
    ERROR_IF(send, sent, == -1);

    n = recv(sv[1], buf, 2, 0);
    ERROR_IF(recv, n, == -1);
    printf("trunc:  %zd %.*s\n", n, (int)n, buf);

    n = recv(sv[1], buf, sizeof(buf), 0);
    ERROR_IF(recv, n, == -1);
    printf("next:   %zd %.*s\n", n, (int)n, buf);

    close(sv[0]);
    close(sv[1]);

    return 0;
}
