#include <signal.h>
#include <stdio.h>
#include <unistd.h>
#include <errno.h>

void handler(int sig) {
    puts("Signal handler called!");
}

int main() {
    if (signal(SIGUSR1, &handler) == SIG_ERR) {
        puts("Signal error!");
        printf("%d\n", errno);
        return 1;
    }

    puts("Raising...");
    if (raise(SIGUSR1)) {
        puts("Raise error!");
        printf("%d\n", errno);
        return 1;
    }
    puts("Raised.");
}
