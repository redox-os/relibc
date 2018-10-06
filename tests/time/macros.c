#include <assert.h>
#include <sys/time.h>

int main() {
    struct timeval x = { .tv_sec = 0, .tv_usec = 15 };
    struct timeval y = { .tv_sec = 0, .tv_usec = 0 };
    struct timeval one = { .tv_sec = 0, .tv_usec = 1 };
    struct timeval max_usec = { .tv_sec = 0, .tv_usec = 999999 };

    assert(!timerisset(&y));
    assert(timerisset(&x));
    timerclear(&x);
    assert(!timerisset(&x));

    assert(timercmp(&x, &y, ==));
    timeradd(&y, &one, &y);
    assert(!timercmp(&x, &y, ==));
    assert(timercmp(&x, &y, <));

    timeradd(&y, &max_usec, &y);
    assert(y.tv_sec == 1);
    assert(y.tv_usec == 0);
    timersub(&y, &one, &y);
    assert(y.tv_sec == 0);
    assert(y.tv_usec == 999999);
}
