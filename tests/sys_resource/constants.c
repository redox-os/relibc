#include <stdio.h>
#include <sys/resource.h>

int main(void) {
    // Checks availability of constants specified in
    // https://pubs.opengroup.org/onlinepubs/7908799/xsh/sysresource.h.html
    printf("PRIO_PROCESS: %d\n", PRIO_PROCESS);
    printf("PRIO_PGRP: %d\n", PRIO_PGRP);
    printf("PRIO_USER: %d\n", PRIO_USER);

    printf("RLIM_INFINITY: %lld\n", RLIM_INFINITY);
    printf("RLIM_SAVED_MAX: %lld\n", RLIM_SAVED_MAX);
    printf("RLIM_SAVED_CUR: %lld\n", RLIM_SAVED_CUR);

    printf("RUSAGE_SELF: %lld\n", RUSAGE_SELF);
    printf("RUSAGE_CHILDREN: %lld\n", RUSAGE_CHILDREN);

    printf("RLIMIT_CORE: %d\n", RLIMIT_CORE);
    printf("RLIMIT_CPU: %d\n", RLIMIT_CPU);
    printf("RLIMIT_DATA: %d\n", RLIMIT_DATA);
    printf("RLIMIT_FSIZE: %d\n", RLIMIT_FSIZE);
    printf("RLIMIT_NOFILE: %d\n", RLIMIT_NOFILE);
    printf("RLIMIT_STACK: %d\n", RLIMIT_STACK);
    printf("RLIMIT_AS: %d\n", RLIMIT_AS);

    return 0;
}
