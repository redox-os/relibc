#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>

void handler(int sig) {
    puts("Signal handler called!");
}

int main(void) {
    if (signal(SIGUSR1, &handler) == SIG_ERR) {
        puts("Signal error!");
        printf("%d\n", errno);
        return EXIT_FAILURE;
    }

    puts("Raising...");
    if (raise(SIGUSR1)) {
        puts("Raise error!");
        printf("%d\n", errno);
        return EXIT_FAILURE;
    }
    puts("Raised.");
}
