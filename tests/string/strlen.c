#include <string.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    char dest1[13] = "hello world!";
    int dest1_len = strlen(dest1);
    printf("%d\n", dest1_len);
    if(dest1_len != 12) {
        puts("strlen(\"hello world!\") failed");
	exit(EXIT_FAILURE);
    }

    char empty[1] = { 0 };
    int empty_len = strlen(empty);
    printf("%d\n", empty_len);
    if(empty_len != 0) {
        puts("strlen(\"\") failed");
        exit(EXIT_FAILURE);
    }

    dest1_len = strnlen(dest1, sizeof(dest1));
    printf("%d\n", dest1_len);
    if(dest1_len != 12) {
        puts("strnlen(\"hello world!\", 13) failed");
        exit(EXIT_FAILURE);
    }

    dest1_len = strnlen(dest1, sizeof(dest1) - 1);
    printf("%d\n", dest1_len);
    if(dest1_len != 12) {
        puts("strnlen(\"hello world!\", 12) failed");
        exit(EXIT_FAILURE);
    }

    dest1_len = strnlen(dest1, 0);
    printf("%d\n", dest1_len);
    if(dest1_len != 0) {
        puts("strnlen(\"hello world!\", 0) failed");
        exit(EXIT_FAILURE);
    }

    dest1_len = strnlen(dest1, 300);
    printf("%d\n", dest1_len);
    if(dest1_len != 12) {
        puts("strnlen(\"hello world!\", 300) failed");
        exit(EXIT_FAILURE);
    }

    dest1_len = strnlen_s(dest1, 6);
    printf("%d\n", dest1_len);
    if(dest1_len != 6) {
        puts("strnlen_s(\"hello world!\", 6) failed");
        exit(EXIT_FAILURE);
    }

    dest1_len = strnlen_s(dest1, 20);
    printf("%d\n", dest1_len);
    if(dest1_len != 12) {
        puts("strnlen_s(\"hello world!\", 20) failed");
        exit(EXIT_FAILURE);
    }

    int null_len = strnlen_s(NULL, 100);
    printf("%d\n", null_len);
    if(null_len != 0) {
        puts("strnlen_s(NULL, 100) failed");
        exit(EXIT_FAILURE);
    }

    return 0;
}
