#include <stdio.h>
#include <stdlib.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>
#include <string.h>
#include <signal.h>
#include <errno.h>

#include "test_helpers.h"

volatile sig_atomic_t signal_received = 0;

void handle_sigusr1(int sig) {
    (void)sig; 
    signal_received = 1;
}

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

        ssize_t n = recv(sv[1], buf, sizeof(buf), 0);
        ERROR_IF(recv, n, == -1);
        printf("child:  %s\n", buf);

        status = send(sv[1], child_msg, 5, 0);
        ERROR_IF(send, status, == -1);

        close(sv[1]);
    } else {
        close(sv[1]); 

        status = send(sv[0], parent_msg, 5, 0);
        ERROR_IF(send, status, == -1);

        ssize_t n = recv(sv[0], buf, sizeof(buf), 0);
        ERROR_IF(recv, n, == -1);
        printf("parent: %s\n", buf);

        close(sv[0]);
        wait(NULL); 
    }

    return 0;
}