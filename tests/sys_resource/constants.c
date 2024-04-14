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

    printf("RLIMIT_CORE: %lld\n", RLIMIT_CORE);
    printf("RLIMIT_CPU: %lld\n", RLIMIT_CPU);
    printf("RLIMIT_DATA: %lld\n", RLIMIT_DATA);
    printf("RLIMIT_FSIZE: %lld\n", RLIMIT_FSIZE);
    printf("RLIMIT_NOFILE: %lld\n", RLIMIT_NOFILE);
    printf("RLIMIT_STACK: %lld\n", RLIMIT_STACK);
    printf("RLIMIT_AS: %lld\n", RLIMIT_AS);

    return 0;
}
