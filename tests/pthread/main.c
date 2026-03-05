#include <assert.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <pthread.h>
#include <fcntl.h>

#include "common.h"

void *thread_main(void *arg) {
    puts("Thread main");

    assert(arg == NULL);

    return NULL;
}

int main(void) {
    int status;

    puts("Start, sleeping 1 second");
    usleep(100000);
    pthread_t thread;
    void *arg = NULL;
    if ((status = pthread_create(&thread, NULL, thread_main, arg)) != 0) {
        return fail(status, "create thread");
    }
    puts("Started");
    void *retval;
    if ((status = pthread_join(thread, &retval)) != 0) {
        return fail(status, "join thread");
    }
    assert(retval == NULL);
    puts("Joined");

    return EXIT_SUCCESS;
}
