#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>

#include "test_helpers.h"

void handler(int sig) {
    puts("Signal handler called!");
}

int main(void) {
    void (*signal_status)(int) = signal(SIGUSR1, handler);
    ERROR_IF(signal, signal_status, == SIG_ERR);
    signal_status = signal(SIGUSR1, handler);
    ERROR_IF(signal, signal_status, != handler);

    puts("Raising...");

    int raise_status = raise(SIGUSR1);
    ERROR_IF(raise, raise_status, < 0);

    puts("Raised.");
}
