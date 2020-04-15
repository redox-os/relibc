#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>

#include "test_helpers.h"

void handler1(int sig) {
    ERROR_IF(handler, sig, != SIGUSR1);
    puts("Signal handler1 called!");
}

void handler2(int sig) {
    ERROR_IF(handler, sig, != SIGUSR1);
    puts("Signal handler2 called!");
}

int main(void) {
	struct sigaction sa1 = { .sa_handler = handler1 };
    struct sigaction sa2 = { .sa_handler = handler2 };
    struct sigaction saold = {0};

	sigemptyset(&sa1.sa_mask);
    sigemptyset(&sa2.sa_mask);

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
