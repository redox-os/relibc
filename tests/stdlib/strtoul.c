#include <errno.h>
#include <stdlib.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    printf("%ld\n", strtoul("         -42", NULL, 0));
    printf("%ld\n", strtoul(" +555", NULL, 0));
    printf("%ld\n", strtoul("   1234567890    ", NULL, 0));

    printf("%ld\n", strtoul("         -42", NULL, 10));
    printf("%ld\n", strtoul(" +555", NULL, 10));
    printf("%ld\n", strtoul("   1234567890    ", NULL, 10));

    printf("%lx\n", strtoul("  0x38Acfg", NULL, 0));
    printf("%lx\n", strtoul("0Xabcdef12", NULL, 16));

    printf("%lo\n", strtoul("  073189", NULL, 0));
    printf("%lo\n", strtoul("     073189", NULL, 8));

    printf("%lo\n", strtoul("  0b", NULL, 8));
    if(errno != 0) {
        printf("errno is not 0 (%d), something went wrong\n", errno);
    }
    printf("%lo\n", strtoul("  0b", NULL, 0));
    if(errno != 0) {
        printf("errno is not 0 (%d), something went wrong\n", errno);
    }
}
