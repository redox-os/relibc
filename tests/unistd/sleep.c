#include <time.h>
#include <unistd.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    // sleep has no error codes and doesn't set errno
    unsigned int unslept = sleep(2);
    printf("unslept: %u\n", unslept);

    int us_status = usleep(1000);
    ERROR_IF(usleep, us_status, == -1);
    UNEXP_IF(usleep, us_status, != 0);

    struct timespec tm = {0, 10000};
    int ns_status = nanosleep(&tm, NULL);
    ERROR_IF(nanosleep, ns_status, == -1);
    UNEXP_IF(nanosleep, ns_status, != 0);
}
