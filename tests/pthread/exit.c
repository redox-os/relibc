#include <assert.h>
#include <stddef.h>
#include <unistd.h>

#include <pthread.h>

#include "common.h"

void *routine(void *arg) {
    assert(arg == NULL);

    usleep(100000);

    puts("Thread succeeded");

    return NULL;
}

int main(void) {
    int status;
    pthread_t thread;

    if ((status = pthread_create(&thread, NULL, routine, NULL)) != 0) {
        return fail(status, "failed to create thread");
    }
    if ((status = pthread_detach(thread)) != 0) {
        return fail(status, "failed to detach thread");
    }

    pthread_exit(NULL);
}
