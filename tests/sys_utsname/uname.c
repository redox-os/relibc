#include <stdio.h>
#include <sys/utsname.h>

int main() {
    struct utsname system_info;

    int result = uname(&system_info);

    if (result < 0) {
        perror("uname");
    } else {
        printf("sysname: '%s'\n", system_info.sysname);
        printf("nodename: '%s'\n", system_info.nodename);
        printf("release: '%s'\n", system_info.release);
        printf("version: '%s'\n", system_info.version);
        printf("machine: '%s'\n", system_info.machine);
        printf("domainname: '%s'\n", system_info.domainname);
    }
}
