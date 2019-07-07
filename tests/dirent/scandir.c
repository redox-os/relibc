#include <dirent.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "test_helpers.h"

int filter(const struct dirent* dirent) {
    return strstr(dirent->d_name, "3") == NULL;
}

int main(void) {
    struct dirent** array;

    int len = scandir("example_dir/", &array, filter, alphasort);
    ERROR_IF(scandir, len, == -1);
    UNEXP_IF(scandir, len, < 0);

    for(int i = 0; i < len; i += 1) {
        // TODO: Redox does not yet provide . or .. - so filter them out
        // in order to make output match on all systems
        if (
            strcmp(array[i]->d_name, ".") != 0 &&
            strcmp(array[i]->d_name, "..") != 0
        ) {
            puts(array[i]->d_name);
        }
        free(array[i]);
    }
    free(array);
}
