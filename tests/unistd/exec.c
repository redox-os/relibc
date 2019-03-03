#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    char* args[] = {"sh", "-c", "echo 'exec works :D'", NULL};

    int status = execv("/bin/sh", args);
    ERROR_IF(execv, status, == -1);
}
