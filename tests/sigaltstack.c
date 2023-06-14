#include <assert.h>
#include <signal.h>
#include <stdlib.h>
#include <stdio.h>

#include "test_helpers.h"

void overflow_stack() {
    overflow_stack();
}

void sigsegv_handler(int sig, siginfo_t *info, void *context) {
    puts("SIGSEGV!");
    _exit(EXIT_FAILURE);
}

int main(void) {
    int status;

    static char STACK[SIGSTKSZ];

    stack_t new, old;

    new.ss_sp = STACK;
    new.ss_size = SIGSTKSZ;
    new.ss_flags = 0;

    struct sigaction action;

    status = sigemptyset(&action.sa_mask);
    ERROR_IF(sigemptyset, status, != 0);

    action.sa_flags = SA_SIGINFO | SA_ONSTACK;
    action.sa_sigaction = sigsegv_handler;

    status = sigaction(SIGSEGV, &action, NULL);
    ERROR_IF(sigaction, status, != 0);

    status = sigaltstack(&new, &old);
    ERROR_IF(sigaltstack, status, != 0);

    assert((old.ss_flags & SS_ONSTACK) == 0);
    assert((old.ss_flags & SS_DISABLE) != 0);

    overflow_stack();
}
