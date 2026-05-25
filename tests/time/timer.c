#include <assert.h>
#include <signal.h>
#include <stdio.h>
#include <sys/types.h>
#include <time.h>

#include "test_helpers.h"

static volatile sig_atomic_t alarm_count = 0;

static void handler(int sig) {
    (void)sig;
    alarm_count++;
}

int main(void) {
    long COUNTDOWN_MILLISECONDS = 100;
    unsigned int SLEEP_MILLISECONDS = 110;

    struct sigaction sa;
    sa.sa_handler = handler;
    sa.sa_flags = 0;
    sigemptyset(&sa.sa_mask);
    int r = sigaction(SIGALRM, &sa, NULL);
    ERROR_IF(sigaction, r, == -1);

    struct sigevent signal_event = {0};
    signal_event.sigev_signo = SIGALRM;
    signal_event.sigev_notify = SIGEV_SIGNAL;

    timer_t timerid = {0};
    int status = 0;
    struct itimerspec current_timer_spec = {0};
    struct itimerspec new_timer_spec = {0};
    new_timer_spec.it_value.tv_sec = 0;
    new_timer_spec.it_value.tv_nsec = COUNTDOWN_MILLISECONDS * 1000000;


    // use an invalid timer
    status = timer_gettime(timerid, &current_timer_spec);
    ERROR_IF(timer_gettime, status, == 0);
    status = timer_settime(timerid, 0, &new_timer_spec, NULL);
    ERROR_IF(timer_settime, status, == 0);
    printf("invalid_timer: ok\n");

    // create a timer
    status = timer_create(CLOCK_MONOTONIC, &signal_event, &timerid);
    ERROR_IF(timer_create, status, == -1);

    printf("timer_create: ok\n");

    // check that no timer has been configured yet
    status = timer_gettime(timerid, &current_timer_spec);
    ERROR_IF(timer_gettime, status, == -1);
    assert(current_timer_spec.it_value.tv_sec == 0);
    assert(current_timer_spec.it_value.tv_nsec == 0);

    printf("timer_gettime: ok\n");

    // start a timer
    status = timer_settime(timerid, 0, &new_timer_spec, &current_timer_spec);
    ERROR_IF(timer_settime, status, == -1);
    // check that there has been no previous timer
    assert(current_timer_spec.it_value.tv_sec == 0);
    assert(current_timer_spec.it_value.tv_nsec == 0);

    // timer_gettime reports the timer
    status = timer_gettime(timerid, &current_timer_spec);
    assert(current_timer_spec.it_value.tv_sec == 0);
    assert(current_timer_spec.it_value.tv_nsec > 0);
    assert(current_timer_spec.it_value.tv_nsec <= COUNTDOWN_MILLISECONDS * 1000000);

    // timer fires
    usleep(SLEEP_MILLISECONDS * 1000);
    assert(alarm_count == 1);

    // timer_gettime reports no timer any more
    status = timer_gettime(timerid, &current_timer_spec);
    assert(current_timer_spec.it_value.tv_sec == 0);
    assert(current_timer_spec.it_value.tv_nsec == 0);

    printf("timer_settime: ok\n");

    // delete the timer
    status = timer_delete(timerid);
    ERROR_IF(timer_delete, status, == -1);

    // any attempts to use the timerid should report EINVAL
    status = timer_gettime(timerid, &current_timer_spec); // must fail
    ERROR_IF(timer_delete, status, == 0);
    assert(errno == EINVAL);
    status = timer_settime(timerid, 0, &new_timer_spec, &current_timer_spec);
    ERROR_IF(timer_delete, status, == 0);
    assert(errno == EINVAL);

    printf("timer_delete: ok\n", status);

    return EXIT_SUCCESS;
}
