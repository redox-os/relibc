#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    FILE *f = freopen("stdio/stdio.in", "r", stdin);
    ERROR_IF(freopen, f, == NULL);

    char in[6];
    fgets(in, 6, stdin);
    printf("%s\n", in); // should print Hello
}
