#include <stdlib.h>
#include <stdio.h>
#include <unistd.h>

int main(void) {
    char hostname[256] = { 0 };

    int status = gethostname(hostname, 256);
    if (status == 0) {
        printf("Hostname: %s\n", hostname);
    } else if (status == -1) {
        perror("gethostname");
        exit(EXIT_FAILURE);
    } else {
        printf("gethostname returned %d, unexpected result\n", status);
        exit(EXIT_FAILURE);
    }
}
