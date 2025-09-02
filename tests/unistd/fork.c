#include <assert.h>
#include <err.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <sys/wait.h>

#include "test_helpers.h"

static void prepare(void) {
    puts("Hello from prepare");
}
static void parent(void) {
    // Make sure we print in the right order and also don't exit
    // before the fork does.
    int us_status = usleep(1000);
    ERROR_IF(usleep, us_status, == -1);
    UNEXP_IF(usleep, us_status, != 0);

    puts("Hello from parent");
}
static void child(void) {
    puts("Hello from child");
}

int main(void) {
    int status = pthread_atfork(prepare, parent, child);
    ERROR_IF(pthread_atfork, status, == -1);

    int pid = fork();
    ERROR_IF(fork, pid, == -1);

    // Avoid zombie processes
    if (pid > 0) {
        int wstatus = 0;
        if (waitpid(pid, &wstatus, 0) == -1) {
            err(EXIT_FAILURE, "Waiting for child process to terminate");
        }

        assert(WIFEXITED(wstatus));
        assert(!WEXITSTATUS(wstatus));
    }

    return EXIT_SUCCESS;
}
