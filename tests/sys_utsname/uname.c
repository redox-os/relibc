#include <stdio.h>
#include <sys/utsname.h>

#include "test_helpers.h"

int main(void) {
    struct utsname system_info;

    int result = uname(&system_info);
    ERROR_IF(uname, result, == -1);
    UNEXP_IF(uname, result, < 0);

    printf("sysname: '%s'\n", system_info.sysname);
    printf("nodename: '%s'\n", system_info.nodename);
    printf("release: '%s'\n", system_info.release);
    printf("version: '%s'\n", system_info.version);
    printf("machine: '%s'\n", system_info.machine);
    //printf("domainname: '%s'\n", system_info.domainname);
}
