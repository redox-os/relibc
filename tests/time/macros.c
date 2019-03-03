#include <assert.h>
#include <sys/time.h>

#include "test_helpers.h"

int main(void) {
    struct timeval x = { .tv_usec = 15 };
    struct timeval y = { 0 };
    struct timeval z = { 0 };
    struct timeval one_usec = { .tv_usec = 1 };
    struct timeval max_usec = { .tv_usec = 999999 };
    struct timeval one_sec = { .tv_sec = 1 };

    assert(!timerisset(&y));
    assert(timerisset(&x));
    timerclear(&x);
    assert(!timerisset(&x));

    assert(timercmp(&x, &y, ==));
    timeradd(&y, &one_usec, &z);
    assert(!timercmp(&x, &z, ==));
    assert(timercmp(&x, &z, <));

    timeradd(&z, &max_usec, &y);
    assert(y.tv_sec == 1);
    assert(y.tv_usec == 0);
    timersub(&y, &one_usec, &z);
    assert(z.tv_sec == 0);
    assert(z.tv_usec == 999999);
    timeradd(&z, &one_sec, &y);
    assert(y.tv_sec == 1);
    assert(y.tv_usec == 999999);
    for (int i = 0; i < 3; i += 1) {
        timersub(&y, &one_sec, &y);
    }
    assert(y.tv_sec == -2);
    assert(y.tv_usec == 999999);
}
