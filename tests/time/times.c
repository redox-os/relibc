#include <stdio.h>
#include <sys/times.h>
#include <unistd.h>

int main() {
    struct tms tms;
    printf("return: %ld\n", times(&tms));

    printf("tm_utime: %ld\n", tms.tms_utime);
    printf("tm_stime: %ld\n", tms.tms_stime);
    printf("tm_cutime: %ld\n", tms.tms_cutime);
    printf("tm_cstime: %ld\n", tms.tms_cstime);
}
