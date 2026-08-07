#include <errno.h>
#include <semaphore.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/types.h>
#include <time.h>
#include <unistd.h>

#include "test_helpers.h"

static sem_t alarm_sem;

static void handler(int sig) {
    (void)sig;
    sem_post(&alarm_sem);
}

int main(void) {
    long COUNTDOWN_MILLISECONDS = 100;
    int r = sem_init(&alarm_sem, 0, 0);
    ERROR_IF(sem_init, r, == -1);

    struct sigaction sa;
    sa.sa_handler = handler;
    sa.sa_flags = 0;
    sigemptyset(&sa.sa_mask);
    r = sigaction(SIGALRM, &sa, NULL);
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
    UNEXP_IF(timer_gettime, current_timer_spec.it_value.tv_sec, != 0);
    UNEXP_IF(timer_gettime, current_timer_spec.it_value.tv_nsec, != 0);

    printf("timer_gettime: ok\n");

    // start a timer
    status = timer_settime(timerid, 0, &new_timer_spec, &current_timer_spec);
    ERROR_IF(timer_settime, status, == -1);
    // check that there has been no previous timer
    UNEXP_IF(timer_settime, current_timer_spec.it_value.tv_sec, != 0);
    UNEXP_IF(timer_settime, current_timer_spec.it_value.tv_nsec, != 0);

    // timer_gettime reports the timer
    status = timer_gettime(timerid, &current_timer_spec);
    UNEXP_IF(timer_gettime, current_timer_spec.it_value.tv_sec, != 0);
    UNEXP_IF(timer_gettime, current_timer_spec.it_value.tv_nsec, == 0);
    // TODO: posix says this can round up, and it happens, why?
    // UNEXP_IF(timer_gettime, current_timer_spec.it_value.tv_nsec, > COUNTDOWN_MILLISECONDS * 1000000);
    
    r = sem_wait(&alarm_sem);
    // will always EINTR because of SIGARLM
    ERROR_IF(sem_wait, r, != -1);
    UNEXP_IF(sem_wait, errno, != EINTR);
    r = sem_wait(&alarm_sem);
    ERROR_IF(sem_wait, r, == -1);

    status = timer_gettime(timerid, &current_timer_spec);
    UNEXP_IF(timer_gettime, current_timer_spec.it_value.tv_sec, != 0);
    UNEXP_IF(timer_gettime, current_timer_spec.it_value.tv_nsec, != 0);

    printf("timer_settime: ok\n");

    // delete the timer
    status = timer_delete(timerid);
    ERROR_IF(timer_delete, status, == -1);

    // any attempts to use the timerid should report EINVAL
    status = timer_gettime(timerid, &current_timer_spec); // must fail
    ERROR_IF(timer_gettime, status, == 0);
    UNEXP_IF(timer_gettime, errno, != EINVAL);
    status = timer_settime(timerid, 0, &new_timer_spec, &current_timer_spec);
    ERROR_IF(timer_settime, status, == 0);
    UNEXP_IF(timer_settime, errno, != EINVAL);

    printf("timer_delete: ok\n");

    sem_destroy(&alarm_sem);

    return EXIT_SUCCESS;
}
