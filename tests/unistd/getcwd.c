#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

int main(void) {
    char first[PATH_MAX];
    getcwd(first, PATH_MAX);
    puts(first);

    char* second = getcwd(NULL, 0);
    puts(second);

    if (strcmp(first, second)) {
        puts("Not matching");
        free(second);
        exit(EXIT_FAILURE);
    }

    free(second);
}
