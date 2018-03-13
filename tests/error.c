#include <unistd.h>
#include <stdio.h>
#include <string.h>
#include <errno.h>

int main(int argc, char** argv) {
    chdir("nonexistent");
    printf("errno: %d = %s\n", errno, strerror(errno));
    perror("perror");
    return 0;
}
