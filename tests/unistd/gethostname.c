#include <stdlib.h>
#include <stdio.h>
#include <unistd.h>

int main() {
    char* hostname = malloc(256);
    if (gethostname(hostname, 256) == 0) {
        printf("Hostname: %s\n", hostname);
    } else {
        puts("error getting hostname");
    }
}
