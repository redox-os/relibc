#include <string.h>
#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    char s0[] = "hello, world";
    char *ptr = strrchr(s0, 'l');
    if (ptr != &s0[10]) {
        printf("%p != %p\n", ptr, &s0[10]);
        puts("strrchr FAIL");
        exit(EXIT_FAILURE);
    }

    char s1[] = "";
    ptr = strrchr(s1, 'a');
    if (ptr != NULL) {
        printf("%p != 0\n", ptr);
        puts("strrchr FAIL");
        exit(EXIT_FAILURE);
    }

    puts("strrch PASS");
}
