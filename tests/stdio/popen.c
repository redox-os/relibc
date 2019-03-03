#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    FILE *fp = popen("ls -1 example_dir", "r");
    ERROR_IF(popen, fp, == NULL);

    char path[256] = { 0 };
    while (fgets(path, 256, fp) != NULL) {
        printf("%s", path);
    }

    int status = pclose(fp);
    ERROR_IF(pclose, status, == -1);
}
