#include <stdio.h>
#include <stdlib.h>
#include <pthread.h>

#include "test_helpers.h"

FILE *f;

void* thread_wontlock(void* arg) {
    (void)arg;

    int result = ftrylockfile(f);
    UNEXP_IF(ftrylockfile, result, == 0);

    return NULL;
}

void* thread_willlock(void* arg) {
    (void)arg;

    int result = ftrylockfile(f);
    UNEXP_IF(ftrylockfile, result, != 0);

    return NULL;
}

int main(void) {
    f = fopen("stdio/stdio.in", "r");
    ERROR_IF(fopen, f, == NULL);
    flockfile(f);
    flockfile(f);
    thread_willlock(NULL);

    pthread_t thread;
    for (int i = 1; i <= 3; i++) {
        pthread_create(&thread, NULL, thread_wontlock, NULL);
        pthread_join(thread, NULL);
        funlockfile(f);
    }

    pthread_create(&thread, NULL, thread_willlock, NULL);
    pthread_join(thread, NULL);

    // TODO: glibc refuses to quit test without this
    //       but in relibc, it will result in EPERM
    // funlockfile(f);
}
