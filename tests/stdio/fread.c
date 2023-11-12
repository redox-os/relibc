#include <errno.h>
#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    FILE *fp = fopen("stdio/fread.in", "rb");
    ERROR_IF(fopen, fp, == NULL);

    char buf[33] = { 0 };
    for (int i = 1; i <= 32; ++i) {
        size_t nread = fread(buf, 1, i, fp);
        if (nread == 0) {
            if (feof(fp)) {
                fprintf(stderr, "early EOF\n");
                return EXIT_FAILURE;
            } else {
                perror("fread");
                return EXIT_FAILURE;
            }
        }
        buf[i] = 0;

        printf("%s\n", buf);
    }

    fclose(fp);
}
