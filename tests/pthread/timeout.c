#include "../test_helpers.h"
#include "common.h"

#include <errno.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

#include <pthread.h>
#include <time.h>

struct arg {
    int status;
    bool completed;
    pthread_barrier_t barrier;
    pthread_mutex_t mutex;
};

void *routine(void *arg_raw) {
    struct arg *arg = (struct arg *)arg_raw;

    int barrier_status = pthread_barrier_wait(&arg->barrier);

    if (barrier_status != 0 && barrier_status != PTHREAD_BARRIER_SERIAL_THREAD) {
        arg->status = barrier_status;
        fputs("failed to wait for barrier\n", stderr);
        return NULL;
    }
    puts("thread waited");

    struct timespec abstime;
    if ((arg->status = clock_gettime(CLOCK_MONOTONIC, &abstime)) != 0) {
        fputs("failed to get current time\n", stderr);
        return NULL;
    }
    abstime.tv_sec += 1;

    if ((arg->status = pthread_mutex_timedlock(&arg->mutex, &abstime)) != ETIMEDOUT) {
        fputs("failed to fail at locking mutex\n", stderr);
        return NULL;
    }
    arg->status = 0;

    return NULL;
}

int main(void) {
    int status;

    struct arg arg;
    arg.completed = false;

    status = pthread_barrier_init(&arg.barrier, NULL, 2);
    ERROR_IF2(pthread_barrier_init, status, != 0);

    status = pthread_mutex_init(&arg.mutex, NULL);
    ERROR_IF2(pthread_mutex_init, status, != 0);

    status = pthread_mutex_trylock(&arg.mutex);
    ERROR_IF2(pthread_mutex_trylock, status, != 0);

    pthread_t thread;
    status = pthread_create(&thread, NULL, routine, &arg);
    ERROR_IF2(pthread_create, status, != 0);

    status = pthread_barrier_wait(&arg.barrier);
    if (status != PTHREAD_BARRIER_SERIAL_THREAD) {
        ERROR_IF2(pthread_barrier_wait, status, != 0);
    }
    puts("main waited");

    status = pthread_join(thread, NULL);
    ERROR_IF2(pthread_join, status, != 0);

    status = pthread_mutex_unlock(&arg.mutex);
    ERROR_IF2(pthread_mutex_unlock, status, != 0);

    if (arg.status != 0) {
        fprintf(stderr, "thread failed: %s\n", strerror(arg.status));
        return EXIT_FAILURE;
    }

    status = pthread_mutex_destroy(&arg.mutex);
    ERROR_IF2(pthread_mutex_destroy, status, != 0);

    status = pthread_barrier_destroy(&arg.barrier);
    ERROR_IF2(pthread_barrier_destroy, status, != 0);

    return EXIT_SUCCESS;
}
