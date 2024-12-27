#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    FILE *fp = popen("ls -1 example_dir", "r");
    ERROR_IF(popen, fp, == NULL);

    int lineno = 0;
    char path[256] = { 0 };
    while (fgets(path, 256, fp) != NULL) {
        lineno++;
        printf("Line %d: %s", lineno, path);
    }

    int status = pclose(fp);
    ERROR_IF(pclose, status, == -1);
}
