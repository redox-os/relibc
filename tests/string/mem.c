#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "test_helpers.h"

int main(void) {
    puts("# mem #");
    char arr[100];
    memset(arr, 0, 100); // Compiler builtin, should work
    arr[50] = 1;
    if ((size_t)memchr((void *)arr, 1, 100) - (size_t)arr != 50) {
        puts("Incorrect memchr");
        exit(EXIT_FAILURE);
    }
    puts("Correct memchr");
    if ((size_t)memrchr((void *)arr, 1, 100) - (size_t)arr != 50) {
        puts("Incorrect memrchr");
        exit(EXIT_FAILURE);
    }
    puts("Correct memrchr");
    char arr2[51];
    memset(arr2, 0, 51); // Compiler builtin, should work
    memccpy((void *)arr2, (void *)arr, 1, 100);
    if (arr[50] != 1) {
        puts("Incorrect memccpy");
        exit(EXIT_FAILURE);
    }
    puts("Correct memccpy");
    int res;
    if ((res = memcmp("hello world", "hello world", 11))) {
        printf("Incorrect memcmp (1), expected 0 found %d\n", res);
        exit(EXIT_FAILURE);
    }
    if ((res = memcmp("hello world", "hello worlt", 11)) >= 0) {
        printf("Incorrect memcmp (2), expected -, found %d\n", res);
        exit(EXIT_FAILURE);
    }
    if ((res = memcmp("hello world", "hallo world", 5)) <= 0) {
        printf("Incorrect memcmp (3), expected +, found %d\n", res);
        exit(EXIT_FAILURE);
    }
    if ((res = memcmp("hello world", "henlo world", 5)) >= 0) {
        printf("Incorrect memcmp (4), expected -, found %d\n", res);
        exit(EXIT_FAILURE);
    }
    puts("Correct memcmp");
}
