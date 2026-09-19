#include <assert.h>
#include <signal.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <setjmp.h>
#include <errno.h>

#include "test_helpers.h"

sigjmp_buf env;

void *fault_addr = (void *)0xdeadbeef;

void segv_handler(int sig, siginfo_t *info, void *context_raw) {
    puts("handling SIGSEGV");

    assert(sig == SIGSEGV);
    assert(info != NULL);
    assert(info->si_signo == SIGSEGV);
    assert(info->si_addr == fault_addr);
    assert(info->si_code == SEGV_MAPERR);
    assert(info->si_status == 0);

    ucontext_t *context = context_raw;
    assert(context != NULL);
    // todo: check context?

    siglongjmp(env, 1);
}

int main(void) {
    struct sigaction sa = { .sa_sigaction = segv_handler, .sa_flags = SA_SIGINFO };
    sigemptyset(&sa.sa_mask);

    int rcode = sigaction(SIGSEGV, &sa, NULL);
    ERROR_IF(sigaction, rcode, != 0);

    if (sigsetjmp(env, 1) == 0) {
        puts("doing SIGSEGV");
        
        volatile int *bad_ptr = (volatile int *)fault_addr;
        *bad_ptr = 42; 

        assert(0); // unreachable
    } else {
        puts("recovered from SIGSEGV");
    }
}
