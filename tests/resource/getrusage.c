#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/resource.h>

#include "test_helpers.h"

void ptimeval(struct timeval* val) {
    printf("{ tv_sec: %ld, tv_usec: %ld }\n", val->tv_sec, val->tv_usec);
}

int main(void) {
    struct rusage r_usage;

    int status = getrusage(RUSAGE_SELF, &r_usage);
    ERROR_IF(getrusage, status, == -1);
    UNEXP_IF(getrusage, status, != 0);

    printf("ru_utime:");
    ptimeval(&r_usage.ru_utime);

    printf("ru_stime:");
    ptimeval(&r_usage.ru_utime);

    printf("ru_maxrss: %ld\n", r_usage.ru_maxrss);
    printf("ru_ixrss: %ld\n", r_usage.ru_ixrss);
    printf("ru_idrss: %ld\n", r_usage.ru_idrss);
    printf("ru_isrss: %ld\n", r_usage.ru_isrss);
    printf("ru_minflt: %ld\n", r_usage.ru_minflt);
    printf("ru_majflt: %ld\n", r_usage.ru_majflt);
    printf("ru_nswap: %ld\n", r_usage.ru_nswap);
    printf("ru_inblock: %ld\n", r_usage.ru_inblock);
    printf("ru_oublock: %ld\n", r_usage.ru_oublock);
    printf("ru_msgsnd: %ld\n", r_usage.ru_msgsnd);
    printf("ru_msgrcv: %ld\n", r_usage.ru_msgrcv);
    printf("ru_nsignals: %ld\n", r_usage.ru_nsignals);
    printf("ru_nvcsw: %ld\n", r_usage.ru_nvcsw);
    printf("ru_nivcsw: %ld\n", r_usage.ru_nivcsw);
}
