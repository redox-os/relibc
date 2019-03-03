#include <stdio.h>
#include <stdlib.h>
#include <errno.h>

#include "test_helpers.h"

int main(void) {
    FILE *f = fopen("stdio/fwrite.out", "w");
    ERROR_IF(fopen, f, == NULL);

    const char ptr[] = "Hello World!";

    if (fwrite(ptr, 0, 17, f)) {
        exit(EXIT_FAILURE);
    }

    if (fwrite(ptr, 7, 0, f)) {
        exit(EXIT_FAILURE);
    }

    if (fwrite(ptr, 0, 0, f)) {
        exit(EXIT_FAILURE);
    }

    fwrite(ptr, sizeof(ptr), 1, f);
    fclose(f);
}
