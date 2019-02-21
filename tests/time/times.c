#include <stdio.h>
#include <sys/times.h>
#include <unistd.h>

#include "test_helpers.h"

int main(void) {
    struct tms tms;

    int status = times(&tms);
    ERROR_IF(times, status, == (time_t)-1);

    printf("tm_utime: %ld\n", tms.tms_utime);
    printf("tm_stime: %ld\n", tms.tms_stime);
    printf("tm_cutime: %ld\n", tms.tms_cutime);
    printf("tm_cstime: %ld\n", tms.tms_cstime);
}
