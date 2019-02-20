#include <unistd.h>
#include <stdio.h>

int main(void) {
    char* args[] = {"sh", "-c", "echo 'exec works :D'", NULL};
    execv("/bin/sh", args);
    perror("execv");
}
