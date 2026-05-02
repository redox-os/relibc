#include <assert.h>
#include <signal.h>
#include <setjmp.h>
#include <unistd.h>
#include <stdio.h>
#include "test_helpers.h"

static sigjmp_buf jmpenv;

void alarm_handler(int sig) {
    (void)sig;
    siglongjmp(jmpenv, 1);
}

int main() {
    struct sigaction sa;

    sa.sa_handler = alarm_handler;
    sigemptyset(&sa.sa_mask);

    int sa_status = sigaction(SIGALRM, &sa, NULL);
    ERROR_IF(sigaction, sa_status, == -1);

    if (sigsetjmp(jmpenv, 1)) {
        printf("SIGALRM interrupted the process.\n");
        return 0;
    }

    alarm(1);
    sleep(5); 

    assert(0); // unreachable
}