#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "test_helpers.h"

int main(void) {
    char *first_null = tmpnam(NULL);
    if(first_null == NULL) {
	// NOTE: assuming that we can at least get one file name
	puts("tmpnam(NULL) returned NULL on first try");
	exit(EXIT_FAILURE);
    }
    printf("%s\n", first_null);

    char *second_null = tmpnam(NULL);
    if(second_null == NULL) {
	// NOTE: assuming that we can at least get one file name
	puts("tmpnam(NULL) returned NULL on second try");
	exit(EXIT_FAILURE);
    }
    printf("%s\n", second_null);

    if(first_null != second_null) {
	puts("tmpnam(NULL) returns different addresses");
	exit(EXIT_FAILURE);
    }

    char buffer[L_tmpnam + 1];
    char *buf_result = tmpnam(buffer);
    if(buf_result == NULL) {
	puts("tmpnam(buffer) failed");
	exit(EXIT_FAILURE);
    } else if(buf_result != buffer) {
	puts("tmpnam(buffer) did not return buffer's address");
	exit(EXIT_FAILURE);
    }
    printf("%s\n", buffer);

    return 0;
}
