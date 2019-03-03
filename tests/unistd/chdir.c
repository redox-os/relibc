#include <limits.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    char cwd[PATH_MAX] = { 0 };
    char *cwd_result = NULL;

    cwd_result = getcwd(cwd, PATH_MAX);
    ERROR_IF(getcwd, cwd_result, == NULL);
    UNEXP_IF(getcwd, cwd_result, != cwd);

    printf("getcwd before chdir: %s\n", cwd);

    int status = chdir("..");
    ERROR_IF(chdir, status, == -1);
    UNEXP_IF(chdir, status, != 0);

    cwd_result = getcwd(cwd, PATH_MAX);
    ERROR_IF(getcwd, cwd_result, == NULL);
    UNEXP_IF(getcwd, cwd_result, != cwd);

    printf("getcwd after chdir: %s\n", cwd);
}
