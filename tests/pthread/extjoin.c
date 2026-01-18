#include <assert.h>
#include <stddef.h>
#include <stdlib.h>
#include <stdio.h>
#include <pthread.h>
#include <unistd.h>

#include "common.h"

struct arg2 {
    int status;
    pthread_t t1;
};

void *routine1(void *arg) {
    assert(arg == NULL);
    puts("Thread 1 spawned, waiting 1s.");
    usleep(100000);
    puts("Thread 1 finished.");
    return strdup("message from thread 1");
}
void *routine2(void *arg_raw) {
    struct arg2 *arg = arg_raw;

    puts("Thread 2 spawned, awaiting thread 1.");

    void *retval_raw;
    int status;

    if ((status = pthread_join(arg->t1, &retval_raw)) != 0) {
        arg->status = fail(status, "t1 join from thread 2");
        return NULL;
    }
    char *retval = retval_raw;

    assert(strcmp(retval, "message from thread 1") == 0);

    free(retval);

    return NULL;
}

int main(void) {
    pthread_t t1;
    pthread_t t2;

    int status;

    puts("Main thread.");

    if ((status = pthread_create(&t1, NULL, routine1, NULL)) != 0) {
        return fail(status, "t1 create");
    }

    puts("Created thread 1.");

    struct arg2 arg = { .status = 0, .t1 = t1 };

    if ((status = pthread_create(&t2, NULL, routine2, &arg)) != 0) {
        return fail(status, "t2 create");
    }

    if ((status = pthread_join(t2, NULL)) != 0) {
        return fail(status, "t2 join");
    }

    return EXIT_SUCCESS;
}
