#include <errno.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "test_helpers.h"

int main(void) {
    char *path_res = realpath("stdlib/realpath.c", NULL);
    ERROR_IF(realpath, path_res, == NULL);
    puts(path_res);
    free(path_res);

    char path_arg[PATH_MAX] = { 0 };
    char *res = realpath("stdlib/realpath.c", path_arg);
    ERROR_IF(realpath, res, == NULL);
    puts(path_arg);
}
