#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "test_helpers.h"

void prepare() {
    puts("Hello from prepare");
}
void parent() {
    // Make sure we print in the right order and also don't exit
    // before the fork does.
    int us_status = usleep(1000);
    ERROR_IF(usleep, us_status, == -1);
    UNEXP_IF(usleep, us_status, != 0);

    puts("Hello from parent");
}
void child() {
    puts("Hello from child");
}

int main(void) {
    int status = pthread_atfork(prepare, parent, child);
    ERROR_IF(pthread_atfork, status, == -1);

    int pid = fork();
    ERROR_IF(fork, pid, == -1);
}
