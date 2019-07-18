#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    FILE *f = fopen("stdio/stdio.in", "r");
    ERROR_IF(fopen, f, == NULL);

    int c = fgetc(f);
    ERROR_IF(fgetc, c, == EOF);
    UNEXP_IF(fgetc, c, < 0);
    UNEXP_IF(fgetc, c, > 255);
    printf("%c\n", c);

    int u = ungetc('J', f);
    ERROR_IF(ungetc, u, == EOF);

    char in[30] = { 0 };
    char *s = fgets(in, 30, f);
    ERROR_IF(fgets, s, == NULL);
    printf("%s\n", in);

    __attribute__((unused)) int vb = setvbuf(stdout, 0, _IONBF, 0);
    //ERROR_IF(setvbuf, vb, > 0); // TODO: Cannot use this, doesn't set errno
    //UNEXP_IF(setvbuf, vb, != 0);

    printf("Hello\n");
}
