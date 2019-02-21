#include <sys/time.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    struct timeval tv;

    // gettimeofday always returns 0, no errors are defined
    int gtod = gettimeofday(&tv, NULL);
    UNEXP_IF(gettimeofday, gtod, != 0);

    printf("%ld: %ld\n", tv.tv_sec, tv.tv_usec);
}
