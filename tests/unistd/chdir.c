#include <limits.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    char cwd[PATH_MAX] = { 0 };
    char *cwd_result = NULL;

    cwd_result = getcwd(cwd, PATH_MAX);
    if (cwd_result == NULL) {
        perror("getcwd");
        exit(EXIT_FAILURE);
    } else if (cwd_result != cwd) {
        puts("getcwd returned something else than the buf argument");
        exit(EXIT_FAILURE);
    }

    printf("getcwd before chdir: %s\n", cwd);

    int status = chdir("..");
    if (status == -1) {
        perror("chdir");
        exit(EXIT_FAILURE);
    } else if (status != 0) {
        printf("chdir returned %d, unexpected result\n", status);
        exit(EXIT_FAILURE);
    }

    cwd_result = getcwd(cwd, PATH_MAX);
    if (cwd_result == NULL) {
        perror("getcwd");
        exit(EXIT_FAILURE);
    } else if (cwd_result != cwd) {
        puts("getcwd returned something else than the buf argument");
        exit(EXIT_FAILURE);
    }

    printf("getcwd after chdir: %s\n", cwd);
}
