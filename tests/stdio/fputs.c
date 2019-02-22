#include <stdio.h>
#include <errno.h>

#include "test_helpers.h"

int main(void) {
    FILE *f = fopen("stdio/fputs.out", "w");
    ERROR_IF(fopen, f, == NULL);

    char *in = "Hello World!";

    int p = fputs(in, f);
    ERROR_IF(fputs, p, == EOF);
    UNEXP_IF(fputs, p, < 0);

    int c = fclose(f);
    ERROR_IF(fclose, c, == EOF);
    UNEXP_IF(fclose, c, != 0);
}
