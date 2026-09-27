#include <stdio.h>
#include <stdlib.h>
#include <signal.h>
#include <unistd.h>
#include <stdint.h>
#include <string.h>
#include "test_helpers.h"

static void *alt_stack_base;
static size_t alt_stack_size = MINSIGSTKSZ;

void segv_handler() {
    int local_var;
    void *current_sp = (void *)&local_var;
    void *alt_stack_end = (char *)alt_stack_base + alt_stack_size;

    if (current_sp >= alt_stack_base && current_sp < alt_stack_end) {
        printf("segv handled in altstack\n");
        exit(0);
    } else {
        printf("segv not handled in altstack\n");
        exit(1);
    }
}

// extra volatile vars to enforce compiler allowing stack overflow
volatile int keep_going = 1;
volatile void *escape_ptr;
void __attribute__((noinline)) overflow_stack() {
    long padding[4096] = {0}; 
    escape_ptr = (void*)padding[0]; 
    if (keep_going) {
        overflow_stack();
    }
}

int main() {
    alt_stack_base = malloc(alt_stack_size);
    UNEXP_IF(malloc, alt_stack_base, == 0);

    stack_t ss = {
        .ss_sp = alt_stack_base,
        .ss_size = alt_stack_size,
        .ss_flags = 0
    };

    int ret = sigaltstack(&ss, NULL);
    ERROR_IF(sigaltstack, ret, == -1);

    struct sigaction sa;
    memset(&sa, 0, sizeof(sa));
    sa.sa_sigaction = segv_handler;
    sa.sa_flags = SA_SIGINFO | SA_ONSTACK; 
    sigemptyset(&sa.sa_mask);

    ret = sigaction(SIGSEGV, &sa, NULL);
    ERROR_IF(sigaction, ret, == -1);

    overflow_stack();

    return 1; // unreachable
}
