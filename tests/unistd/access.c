#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    if (access("example_dir/1-never-gonna-give-you-up", R_OK | W_OK)) {
        perror("access");
        exit(EXIT_FAILURE);
    }
    if (!access("example_dir/1-never-gonna-give-you-up", X_OK)) {
        puts("Accessing a file with X_OK worked even though it... probably... shouldn't?");
        puts("Please run `chmod 644 example_dir/*` and try again.");
        exit(EXIT_FAILURE);
    }
}
