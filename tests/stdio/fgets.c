#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    FILE *f = fopen("stdio/stdio.in", "r");
    ERROR_IF(fopen, f, == NULL);

    char line[256];

    while (1) {
        if (fgets(line, 256, f)) {
            fputs(line, stdout);
        } else {
            puts("EOF");
            if (!feof(f)) {
                puts("feof() not updated!");
                exit(EXIT_FAILURE);
            }
            break;
        }
    }
}
