#include <assert.h>
#include <signal.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>

#include "test_helpers.h"

void handler1(int sig) {
    assert(sig == SIGUSR1);
    char *str = "Signal handler1 called!\n";
    write(STDOUT_FILENO, str, strlen(str));
}

sigset_t the_set = { 0 };

void handler2(int sig, siginfo_t *info, void *context_raw) {
    assert(sig == SIGUSR1);
    char *str = "Signal handler2 called!\n";
    write(STDOUT_FILENO, str, strlen(str));

    assert(info != NULL);
    assert(info->si_signo == SIGUSR1);
#ifndef __linux
    // TODO: SI_TKILL?
    assert(info->si_code == SI_USER);
    assert(info->si_pid == getpid());
    assert(info->si_uid == getuid());
#endif

    ucontext_t *context = context_raw;
    assert(context != NULL);
#ifndef __linux__ // TODO
    assert(memcmp(&context->uc_sigmask, &the_set, sizeof(sigset_t)));
    assert(context->uc_link == NULL);
#endif
}

int main(void) {
	struct sigaction sa1 = { .sa_handler = handler1 };
    struct sigaction sa2 = { .sa_sigaction = handler2, .sa_flags = SA_SIGINFO };
    struct sigaction saold = {0};

	sigemptyset(&sa1.sa_mask);
    sigemptyset(&sa2.sa_mask);

    int status = sigprocmask(SIG_SETMASK, NULL, &the_set);
    ERROR_IF(sigprocmask, status, == -1);

    int rcode = sigaction(SIGUSR1, &sa1, NULL);
    ERROR_IF(signal, rcode, != 0);

    puts("Raising...");

    int raise_status = raise(SIGUSR1);
    ERROR_IF(raise, raise_status, < 0);

	rcode = sigaction(SIGUSR1, &sa2, &saold);
    ERROR_IF(signal, rcode, != 0);
    ERROR_IF(signal, saold.sa_handler, != sa1.sa_handler);

    puts("Raising...");

    raise_status = raise(SIGUSR1);
    ERROR_IF(raise, raise_status, < 0);

    puts("Raised.");
}
