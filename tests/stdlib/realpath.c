#include <errno.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    char* path = realpath("stdlib/realpath.c", NULL);
    if (!path) {
        perror("realpath");
        exit(EXIT_FAILURE);
    }
    puts(path);

    free(path);

    path = malloc(PATH_MAX);
    memset(path, 0, PATH_MAX);

    realpath("stdlib/realpath.c", path);
    if (!path) {
        perror("realpath");
        free(path);
        exit(EXIT_FAILURE);
    }
    puts(path);

    free(path);
}
