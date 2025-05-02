#include <errno.h>
#include <stdlib.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    char *endptr;

    printf("%ld\n", strtol("         -42", &endptr, 0));
    printf("endptr \"%s\"\n", endptr);
    printf("%ld\n", strtol(" +555", &endptr, 0));
    printf("endptr \"%s\"\n", endptr);
    printf("%ld\n", strtol("   1234567890    ", &endptr, 0));
    printf("endptr \"%s\"\n", endptr);

    printf("%ld\n", strtol("         -42", &endptr, 10));
    printf("endptr \"%s\"\n", endptr);
    printf("%ld\n", strtol(" +555", &endptr, 10));
    printf("endptr \"%s\"\n", endptr);
    printf("%ld\n", strtol("   1234567890    ", &endptr, 10));
    printf("endptr \"%s\"\n", endptr);

    printf("%lx\n", strtol("  0x38Acfg", &endptr, 0));
    printf("endptr \"%s\"\n", endptr);
    printf("%lx\n", strtol("0Xabcdef12", &endptr, 16));
    printf("endptr \"%s\"\n", endptr);
    printf("%lx\n", strtol("cafebeef", &endptr, 16));
    printf("endptr \"%s\"\n", endptr);

    printf("%lo\n", strtol("  073189", &endptr, 0));
    printf("endptr \"%s\"\n", endptr);
    printf("%lo\n", strtol("     073189", &endptr, 8));
    printf("endptr \"%s\"\n", endptr);

    printf("%lo\n", strtol("  0b", &endptr, 8));
    if(errno != 0) {
        printf("errno is not 0 (%d), something went wrong\n", errno);
    }
    printf("endptr \"%s\"\n", endptr);
    printf("%lo\n", strtol("  0b", &endptr, 0));
    if(errno != 0) {
        printf("errno is not 0 (%d), something went wrong\n", errno);
    }
    printf("endptr \"%s\"\n", endptr);
}
