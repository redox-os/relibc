#include <time.h>
#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    struct timespec tm = {0, 0};

    int cgt = clock_gettime(CLOCK_REALTIME, &tm);
    ERROR_IF(clock_gettime, cgt, == -1);

    time_t t = time(NULL);
    ERROR_IF(time, t, == (time_t)-1);

    // TODO: Support clock() on Redox
    // clock_t c = clock();
    // ERROR_IF(clock, c, == (clock_t)-1);
}
