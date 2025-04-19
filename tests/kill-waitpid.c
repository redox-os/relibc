#include <assert.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include <sys/wait.h>

#include "test_helpers.h"

int main(void) {
    int err;

    int fds[2];
    err = pipe(fds);
    ERROR_IF(pipe, err, == -1);

    int child = fork();
    ERROR_IF(fork, child, == -1);

    if (child == 0) {
        // block forever
        char buf[1];
        read(fds[0], buf, 1);
    }

    err = kill(child, SIGTERM);
    ERROR_IF(kill, err, == -1);

    int status;
    err = waitpid(child, &status, 0);
    ERROR_IF(waitpid, err, == -1);
    printf("STATUS %d\n", status);
    assert(WIFSIGNALED(status));
    assert(WTERMSIG(status) == SIGTERM);
}
