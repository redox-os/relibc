#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    FILE *f = fopen("stdio/stdio.in", "r");
    ERROR_IF(fopen, f, == NULL);

    int status = fseek(f, 14, SEEK_CUR);
    ERROR_IF(fseek, status, == -1);
    UNEXP_IF(fseek, status, != 0);

    char buffer[256];
    printf("%s", fgets(buffer, 256, f));

    off_t pos = ftello(f);
    ERROR_IF(ftello, pos, == -1);
    printf("ftello: %ld\n", pos);
}
