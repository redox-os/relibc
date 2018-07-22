#include <unistd.h>
#include <stdio.h>

int main(int argc, char** argv) {
    char* args[] = {"sh", "-c", "echo 'exec works :D'", NULL};
    execv("/bin/sh", args);
    perror("execv");
    return 0;
}
