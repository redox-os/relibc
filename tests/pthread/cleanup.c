#include <assert.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <pthread.h>

const char *msg1 = "first";
const char *msg2 = "second";
const char *msg3 = "third";

void cleanup1(void *arg) {
    printf("Running %s cleanup callback\n", (const char *)arg);
}
void cleanup2(void *arg) {
    fprintf(stderr, "Running %s cleanup callback, to stderr\n", (const char *)arg);
}
void cleanup3(void *arg) {
    printf("Running final (%s) callback\n", (const char *)arg);
}

void *routine(void *arg) {
    assert(arg == NULL);

    puts("1");
    pthread_cleanup_push(cleanup1, msg1);
    puts("2");
    pthread_cleanup_push(cleanup2, msg2);
    puts("3");
    pthread_cleanup_push(cleanup3, msg3);
    puts("4");
    pthread_cleanup_pop(true);
    puts("5");
    //exit(EXIT_SUCCESS);
    pthread_exit(NULL);
    puts("6");
    pthread_cleanup_pop(true);
    pthread_cleanup_pop(true);
    return NULL;
}

int main(void) {
    int result;

    puts("Main thread started");
    pthread_t second_thread;
    if ((result = pthread_create(&second_thread, NULL, routine, NULL)) != 0) {
        fprintf(stderr, "thread creation failed: %s\n", strerror(result));
        return EXIT_FAILURE;
    }
    if ((result = pthread_join(second_thread, NULL)) != 0) {
        fprintf(stderr, "failed to join thread: %s\n", strerror(result));
        return EXIT_FAILURE;
    }
    puts("Main thread about to exit");
    return EXIT_SUCCESS;
}
