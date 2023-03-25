#include <stdio.h>
#include <string.h>

#include "test_helpers.h"

int main(void) {
    char dst[20];

    strcpy(dst, "strcpy works!");
    puts(dst);
    strncpy(dst, "strncpy works!", 20);
    puts(dst);

    // Make sure no NUL is placed
    memset(dst, 'a', 20);
    dst[19] = 0;
    strncpy(dst, "strncpy should work here too", 10);
    puts(dst);

    // The string should be properly terminated regardless
    char ndst[28];

    size_t r = strlcpy(ndst, "strlcpy works!", 28);
    puts(ndst);
    printf("copied %lu\n", r);
    r = strlcat(ndst, " and strlcat!", 28);
    puts(ndst);
    printf("copied %lu\n", r);
}
