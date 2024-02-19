#include <stdio.h>
#include <unistd.h>
#include <errno.h>

#include "test_helpers.h"

int main(void) {
    int getpagesize_result = getpagesize();

    printf("Page size: %d\n", getpagesize_result);
}
