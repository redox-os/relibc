#include <assert.h>
#include <errno.h>
#include <signal.h>
#include <sched.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/wait.h>
#include <unistd.h>

#include "test_helpers.h"

volatile sig_atomic_t atomic = 0;
volatile sig_atomic_t atomic2 = 0;

void action(int sig, siginfo_t *info, void *context) {
    (void)context;

    assert(sig == SIGCHLD);
    assert(info != NULL);
    atomic += 1;
}
void handler(int sig) {
    assert(sig == SIGPIPE);
    atomic2 += 1;
}

int main(void) {
    int child = fork();
    ERROR_IF(fork, child, == -1);

    int fds[2];
    int status = pipe(fds);
    ERROR_IF(pipe, status, == -1);

    struct sigaction sa;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = 0;
    sa.sa_sigaction = action;
    status = sigaction(SIGCHLD, &sa, NULL);
    ERROR_IF(sigaction, status, == -1);

    sa.sa_handler = handler;
    status = sigaction(SIGPIPE, &sa, NULL);
    ERROR_IF(sigaction, status, == -1);

    if (child == 0) {
        status = close(fds[1]);
        ERROR_IF(close, status, == -1);

        char buf[1];
        status = read(fds[0], buf, 1);
        ERROR_IF(read, status, == -1);
        puts("Child exiting.");
        return EXIT_SUCCESS;
    } else {
        int waitpid_stat;

        close(fds[0]);
        ERROR_IF(close, status, == -1);

        puts("Sending SIGSTOP...");
        status = kill(child, SIGSTOP);
        ERROR_IF(kill, status, == -1);

        while (atomic == 0) {
            status = sched_yield();
            ERROR_IF(sched_yield, status, == -1);
        }
        puts("First handler ran, checking status.");

        status = waitpid(child, &waitpid_stat, WUNTRACED);
        ERROR_IF(waitpid, status, == -1);
        assert(WIFSTOPPED(waitpid_stat));
        assert(WSTOPSIG(waitpid_stat) == SIGSTOP);
        puts("Correct, sending SIGCONT...");

        status = kill(child, SIGCONT);
        ERROR_IF(kill, status, == -1);

        while (atomic == 1) {
            status = sched_yield();
            ERROR_IF(sched_yield, status, == -1);
        }
        puts("Second handler ran, checking status.");

        status = waitpid(child, &waitpid_stat, 0);
        ERROR_IF(waitpid, status, == -1);
        assert(WIFEXITED(waitpid_stat));
        assert(WEXITSTATUS(waitpid_stat) == 0);
        puts("Child exited.");

        puts("Writing to (broken) pipe.");
        status = write(fds[1], "C", 1);
        ERROR_IF(write, status, != -1);
        assert(errno == EPIPE);

        while (atomic2 == 0) {
            status = sched_yield();
            ERROR_IF(sched_yield, status, == -1);
        }
        puts("SIGSTOP handler successfully executed.");
        return EXIT_SUCCESS;
    }
}
