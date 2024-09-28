#include <assert.h>
#include <signal.h>
#include <stdio.h>
#include <unistd.h>
#include <errno.h>
#include <sys/mman.h>

#include "test_helpers.h"

size_t stack_size;
void *stack_base;

volatile sig_atomic_t counter = 0;

void action(int sig, siginfo_t *info, void *context_raw) {
    assert(sig == SIGUSR2);

    ucontext_t *context = context_raw;
    assert(context->uc_stack.ss_sp == stack_base);
    assert(context->uc_stack.ss_size == stack_size);

    // TODO: Technically an implementation detail, but safe to check here.
    assert((size_t)info >= (size_t)stack_base);
    assert((size_t)info <= ((size_t)stack_base + stack_size));

    int c = counter++;

    char str[] = "SIGUSR2 handlerXX\n";
    size_t len = strlen(str);
    str[len - 2] = '0' + (c % 10);
    str[len - 3] = '0' + ((c / 10) % 10);
    write(STDOUT_FILENO, str, len);

    if (c < 100) {
        raise(SIGUSR2);
    }
}

int main(void) {
    int status;

    stack_size = 1024 * 1024; // TODO?
    assert(stack_size >= MINSIGSTKSZ * 100);
    stack_base = mmap(NULL, stack_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    ERROR_IF(mmap, stack_base, == MAP_FAILED);

    stack_t old_stack;
    stack_t stack = (stack_t) { .ss_sp = stack_base, .ss_size = stack_size, .ss_flags = 0 };

    status = sigaltstack(&stack, &old_stack);
    ERROR_IF(sigaltstack, status, == -1);

    assert((old_stack.ss_flags & SS_ONSTACK) == 0);

    stack_t same;

    status = sigaltstack(&old_stack, &same);
    ERROR_IF(sigaltstack, status, == -1);
    assert(same.ss_sp == stack.ss_sp);
    assert(same.ss_size == stack.ss_size);
    assert((same.ss_flags & SS_ONSTACK) == 0);

    status = sigaltstack(&stack, NULL);
    ERROR_IF(sigaltstack, status, == -1);

    status = sigaltstack(NULL, &same);
    ERROR_IF(sigaltstack, status, == -1);
    assert(same.ss_sp == stack.ss_sp);
    assert(same.ss_size == stack.ss_size);
    assert((same.ss_flags & SS_ONSTACK) == 0);

    struct sigaction sa;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = SA_ONSTACK | SA_SIGINFO | SA_NODEFER;
    sa.sa_sigaction = action;

    status = sigaction(SIGUSR2, &sa, NULL);
    ERROR_IF(sigaction, status, == -1);

    raise(SIGUSR2);

    return 0;
}
