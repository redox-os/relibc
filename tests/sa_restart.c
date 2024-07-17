#include <assert.h>
#include <signal.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>

#include "test_helpers.h"

void action(int sig, siginfo_t *info, void *context) {
    assert (sig == SIGUSR1);
    (void)info;
    (void)context;
    char *msg = "Signal handler\n";
    write(1, msg, strlen(msg));
    _exit(0);
}

int main(void) {
    int status;

    struct sigaction act;
    act.sa_sigaction = action;
    act.sa_flags = SA_RESTART;
    sigemptyset(&act.sa_mask);

    status = sigaction(SIGUSR1, &act, NULL);
    ERROR_IF(sigaction, status, == -1);

    int fds[2];
    status = pipe(fds);
    ERROR_IF(pipe, status, == -1);

    int parent = getpid();

    status = fork();
    ERROR_IF(fork, status, == -1);

    if (status == 0) {
        for (int i = 0; i < 1000; i++) {
            status = kill(parent, SIGUSR1);
            ERROR_IF(kill, status, == -1);
        }
        return 0;
    }

    char buffer[1];
    status = read(fds[0], buffer, 1);
    ERROR_IF(read, status, == -1);

    return 1;
}
