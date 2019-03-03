#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "test_helpers.h"

int main(void) {
    int status = isatty(STDOUT_FILENO);

    if (status == 1) {
        puts("'Tis a tty :D");
    } else if (status == 0) {
        if (errno == ENOTTY) {
            // I wouldn't consider stdout not being a TTY an error
            // (CI runners, etc.) 
            puts("Whatever a tty is, it's not me");
        } else {
            perror("isatty");
            exit(EXIT_FAILURE);
        }
    } else {
        printf("isatty returned %d, unexpected result\n", status);
        exit(EXIT_FAILURE);
    }
}
