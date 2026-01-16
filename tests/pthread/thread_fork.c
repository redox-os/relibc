/// test fork inside thread

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <pthread.h>
#include <sys/wait.h>

#include "test_helpers.h"

#define NUM_THREADS 4
#define ITERATIONS 3


void* thread_func(void* arg) {
    long tid = (long)arg;

    for (int i = 0; i < ITERATIONS; i++) {
        printf("thread %ld loop %i\n", tid, i);

        pid_t pid = fork();
        ERROR_IF(fork, pid, < 0);

        if (pid == 0) {
            _exit(0);
        } else {
            waitpid(pid, NULL, 0);
        }

        usleep(1000); 
    }

    return NULL;
}

int main() {
    pthread_t threads[NUM_THREADS];
    int status;

    for (long i = 0; i < NUM_THREADS; i++) {
        status = pthread_create(&threads[i], NULL, thread_func, (void*)i);
        ERROR_IF(pthread_create, status, != 0);
    }

    printf("joining threads\n");
    for (int i = 0; i < NUM_THREADS; i++) {
        pthread_join(threads[i], NULL);
    }

    return 0;
}
