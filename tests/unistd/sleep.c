#include <time.h>
#include <unistd.h>
#include <stdio.h>
#include <signal.h>

#include "test_helpers.h"

int usleep(useconds_t);

void handler(int _s)
{
    (void)_s;
}

int main(void)
{
    // sleep has no error codes and doesn't set errno
    unsigned int unslept = sleep(1);
    printf("unslept: %u\n", unslept);

    struct sigaction sa;
    memset(&sa, 0, sizeof(sa));
    sa.sa_handler = handler;
    sigaction(SIGALRM, &sa, NULL);
    // TODO: This test is unreliable, overlapping use of alarm and sleep is not recommended.
    // alarm(1);

    // unslept = sleep(10);
    // if (unslept < 7)
    // {
    //     printf("after alarm, unslept too short: %u\n", unslept);
    //     exit(EXIT_FAILURE);
    // }

    int us_status = usleep(100);
    ERROR_IF(usleep, us_status, == -1);
    UNEXP_IF(usleep, us_status, != 0);

    struct timespec tm = {0, 10000};
    int ns_status = nanosleep(&tm, NULL);
    ERROR_IF(nanosleep, ns_status, == -1);
    UNEXP_IF(nanosleep, ns_status, != 0);
}
