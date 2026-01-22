#include <stdio.h>
#include <stdlib.h>
#include <pthread.h>
#include <time.h>
#include <errno.h>
#include <unistd.h>

#include "test_helpers.h"

pthread_mutex_t lock;
pthread_cond_t cond;
struct timespec start_time;
int ready = 0;

struct timespec get_timeout(long msec) {
    struct timespec ts;
    clock_gettime(CLOCK_REALTIME, &ts);
    ts.tv_nsec += msec * 1000000;
    return ts;
}

void print_timed(char* msg) {
    struct timespec end_time;
    clock_gettime(CLOCK_MONOTONIC, &end_time);
    printf("[%.3f]: ", (end_time.tv_nsec - start_time.tv_nsec) / 1000000000.0 +
            (end_time.tv_sec  - start_time.tv_sec));
    printf("%s\n", msg);
    fflush(NULL);
}

void* signaler_thread(void* arg) {
    (void)arg;

    usleep(150000); // 150ms
    ready = 1;
    print_timed("signaler_thread");
    pthread_cond_signal(&cond);
    return NULL;
}

void test_timeout_case() {
    pthread_mutex_lock(&lock);
    struct timespec ts = get_timeout(500);
    print_timed("test_timeout_case start");
    int rc = pthread_cond_timedwait(&cond, &lock, &ts);
    UNEXP_IF(pthread_cond_timedwait, rc, != ETIMEDOUT);
    print_timed("test_timeout_case end");
    pthread_mutex_unlock(&lock);
}

void test_success_case() {
    ready = 0;
    pthread_t thread;
    pthread_create(&thread, NULL, signaler_thread, NULL);
    pthread_mutex_lock(&lock);
    struct timespec ts = get_timeout(50000);
    print_timed("test_success_case start");
    while (!ready) {
        print_timed("test_success_case waiting");
        int rc = pthread_cond_timedwait(&cond, &lock, &ts);
        UNEXP_IF(pthread_cond_timedwait, rc, != 0);
    }
    print_timed("test_success_case end");
    pthread_mutex_unlock(&lock);
    pthread_join(thread, NULL);
}

int main() {
   clock_gettime(CLOCK_MONOTONIC, &start_time);
    if (pthread_mutex_init(&lock, NULL) != 0) {
        perror("mutex init failed");
        return 1;
    }
    if (pthread_cond_init(&cond, NULL) != 0) {
        perror("cond init failed");
        return 1;
    }
    test_timeout_case();
    // TODO: "rlct_clone not implemented for aarch64 yet"
#if defined(__linux__) && defined(__aarch64__)
    printf("test_success_case skipped");
#else
    test_success_case();
#endif
    return 0;
}
