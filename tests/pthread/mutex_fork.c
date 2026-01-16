/// test mutex inside fork

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <pthread.h>
#include <sys/wait.h>

#include "test_helpers.h"

#define NUM_THREADS 4
#define ITERATIONS 3

pthread_mutex_t global_lock;

void* thread_func(void* arg) {
    long tid = (long)arg;

    for (int i = 0; i < ITERATIONS; i++) {
        printf("thread %ld loop %i lock\n", tid, i);
        
        pthread_mutex_lock(&global_lock);
        
        pid_t pid = fork();

        if (pid < 0) {
            perror("Fork failed");
            pthread_mutex_unlock(&global_lock);
            pthread_exit(NULL);
        }

        if (pid == 0) {
            _exit(0);
        } else {
            printf("thread %ld unlock\n", tid);
            pthread_mutex_unlock(&global_lock);

            waitpid(pid, NULL, 0);
        }
        
        usleep(1000); 
    }

    return NULL;
}

int main() {
    pthread_t threads[NUM_THREADS];
    int status;
    pthread_mutex_init(&global_lock, NULL);

    for (long i = 0; i < NUM_THREADS; i++) {
        status = pthread_create(&threads[i], NULL, thread_func, (void*)i);
        ERROR_IF(pthread_create, status, != 0);
    }

    for (int i = 0; i < NUM_THREADS; i++) {
        pthread_join(threads[i], NULL);
    }

    pthread_mutex_destroy(&global_lock);
    return 0;
}
