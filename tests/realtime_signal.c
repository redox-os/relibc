#include "test_helpers.h"

#include <assert.h>
#include <errno.h>
#include <pthread.h>
#include <signal.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

int fds[2];

struct order {
    const char *msg;
    bool handler2_not_handler1;
} order[3];
unsigned count = 0;

void *thread_start(void *arg_raw) {
    pthread_barrier_t *barrier = arg_raw;

    int status;

    status = pthread_barrier_wait(barrier);
    if (status != PTHREAD_BARRIER_SERIAL_THREAD) {
        ERROR_IF(pthread_barrier_wait, status, != 0);
    }
    puts("Thread waited for barrier.");

    sigset_t mask;
    status = sigemptyset(&mask);
    ERROR_IF(sigemptyset, status, != 0);

    status = pthread_sigmask(SIG_SETMASK, &mask, NULL);
    ERROR_IF(pthread_sigmask, status, != 0);

    // By now, the three queued signals will be sent. signal_number1 will be
    // sent first, FIFO, as specified by POSIX. signal_number2 will be sent
    // last.

    unsigned char buf;

    while (count != 3) {
        status = read(fds[0], &buf, 1);
        ERROR_IF(read, status, != 0);
    }

    return NULL;
}
void handler1(int sig, siginfo_t *info, void *context) {
    order[count++] = (struct order) { .msg = info->si_value.sigval_ptr, .handler2_not_handler1 = false };
    put_string(info->si_value.sigval_ptr);
}
void handler2(int sig, siginfo_t *info, void *context) {
    order[count++] = (struct order) { .msg = info->si_value.sigval_ptr, .handler2_not_handler1 = true };
    put_string(info->si_value.sigval_ptr);
}
void put_string(const char *str) {
    size_t len = strlen(str);
    write(STDERR_FILENO, str, len);
}

#define MSG0 "Calling signal 2\n"
#define MSG1 "Calling signal 1, first time\n"
#define MSG2 "Calling signal 1, second time\n"

const char *expected_msgs[3] = { MSG1, MSG2, MSG0 };

int main(void) {
    int status;

    status = pipe(fds);
    ERROR_IF(pipe, status, != 0);

    int signal_number1 = SIGRTMIN;
    int signal_number2 = SIGRTMIN + 1;

    sigset_t mask;

    status = sigemptyset(&mask);
    ERROR_IF(sigemptyset, status, != 0);

    status = sigaddset(&mask, signal_number1);
    ERROR_IF(sigaddset, status, != 0);

    status = sigaddset(&mask, signal_number2);
    ERROR_IF(sigaddset, status, != 0);

    status = pthread_sigmask(SIG_SETMASK, &mask, NULL);
    ERROR_IF(pthread_sigmask, status, != 0);

    struct sigaction act;
    memcpy(&act.sa_mask, &mask, sizeof(sigset_t));
    act.sa_flags = SA_SIGINFO;
    act.sa_sigaction = handler1;

    status = sigaction(signal_number1, &act, NULL);
    ERROR_IF(sigaction, status, != 0);

    act.sa_sigaction = handler2;

    status = sigaction(signal_number2, &act, NULL);
    ERROR_IF(sigaction, status, != 0);

    pthread_barrier_t barrier;
    status = pthread_barrier_init(&barrier, NULL, 2);
    ERROR_IF(pthread_barrier_init, status, != 0);

    pthread_t thread;
    status = pthread_create(&thread, NULL, thread_start, &barrier);

    ERROR_IF(pthread_create, status, != 0);

    union sigval sigval;
    sigval.sigval_ptr = MSG0;

    status = pthread_sigqueue(thread, signal_number2, sigval);
    ERROR_IF(pthread_sigqueue, status, != 0);

    sigval.sigval_ptr = MSG1;

    status = pthread_sigqueue(thread, signal_number1, sigval);
    ERROR_IF(pthread_sigqueue, status, != 0);

    sigval.sigval_ptr = MSG2;

    status = pthread_sigqueue(thread, signal_number1, sigval);
    ERROR_IF(pthread_sigqueue, status, != 0);

    status = pthread_barrier_wait(&barrier);
    if (status != PTHREAD_BARRIER_SERIAL_THREAD) {
        ERROR_IF(pthread_barrier_wait, status, != 0);
    }

    puts("Main waited for barrier.");

    status = pthread_join(thread, NULL);
    ERROR_IF(pthread_join, status, != 0);

    status = pthread_barrier_destroy(&barrier);
    ERROR_IF(pthread_barrier_destroy, status, != 0);

    for (size_t i = 0; i < 3; i++) {
        struct order j = order[i];

        assert(strcmp(j.msg, expected_msgs[i]) == 0);
    }
    assert(!order[0].handler2_not_handler1);
    assert(!order[1].handler2_not_handler1);
    assert(order[2].handler2_not_handler1);

    return 0;
}
