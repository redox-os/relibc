#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "test_helpers.h"

int main(void) {
    char buffer[72];

    int ret = sprintf(
        buffer,
        "This string fits in the buffer because it is only %d bytes in length",
        68
    );
    if (ret != 68) {
        printf("Failed! Return value was %d\n", ret);
        exit(EXIT_FAILURE);
    }

    memset(buffer, 0, sizeof(buffer));

    ret = snprintf(
        buffer,
        10,
        "This string is way longer and does not fit in the buffer because it %d bytes in length",
        86
    );
    if (ret != 86) {
        printf("Failed! Return value was %d\n", ret);
        exit(EXIT_FAILURE);
    }

    puts(buffer);
}
