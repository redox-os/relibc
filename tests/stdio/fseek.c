#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    FILE *f = fopen("stdio/stdio.in", "r");
    ERROR_IF(fopen, f, == NULL);

    if (fseek(f, 14, SEEK_CUR) < 0) {
        puts("fseek error");
        exit(EXIT_FAILURE);
    }
    char buffer[256];
    printf("%s", fgets(buffer, 256, f));
    printf("ftell: %ld\n", ftello(f));
}
