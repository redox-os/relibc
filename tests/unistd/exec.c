#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    char* args[] = {"sh", "-c", "echo 'exec works :D'", NULL};

    int status = execv("/bin/sh", args);
    if (status == -1) {
        perror("execv");
        exit(EXIT_FAILURE);
    }
}
