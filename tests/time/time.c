#include <err.h>
#include <stdlib.h>
#include <time.h>

#include "test_helpers.h"

int main(void) {
    struct timespec tm = {0};

    int cgt = clock_gettime(CLOCK_REALTIME, &tm);
    ERROR_IF(clock_gettime, cgt, == -1);

    cgt = clock_getres(CLOCK_REALTIME, &tm);
    ERROR_IF(clock_getres, cgt, == -1);

    cgt = timespec_get(&tm, TIME_UTC);
    if (cgt != TIME_UTC) {
        errx(
            EXIT_FAILURE,
            "timespec_get should have returned %d but returned %d\n",
            TIME_UTC,
            cgt
        );
    }

    cgt = timespec_getres(&tm, TIME_UTC);
    if (cgt != TIME_UTC) {
        errx(
            EXIT_FAILURE,
            "timespec_getres should have returned %d but returned %d\n",
            TIME_UTC,
            cgt
        );
    }

    time_t t = time(NULL);
    ERROR_IF(time, t, == (time_t)-1);

    // TODO: Support clock() on Redox
    // clock_t c = clock();
    // ERROR_IF(clock, c, == (clock_t)-1);

    return EXIT_SUCCESS;
}
