#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <signal.h>

#include "test_helpers.h"

int main(void) {
    puts("# strsignal #");
    const char *x = strsignal(SIGHUP);
    if (strcmp(x, "Hangup")) {
        printf("Incorrect strsignal (1), found: .%s.\n", x);
        exit(EXIT_FAILURE);
    }
    x = strsignal(0); 
    if (strcmp(x, "Unknown signal")) {
        printf("Incorrect strsignal (2), found: .%s.\n", x);
        exit(EXIT_FAILURE);
    }
    x = strsignal(100); 
    if (strcmp(x, "Unknown signal")) {
        printf("Incorrect strsignal (3), found: .%s.\n", x);
        exit(EXIT_FAILURE);
    }


}
