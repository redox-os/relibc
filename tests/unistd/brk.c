#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    int status = brk((void*)100);

    if (status == -1) {
        perror("brk");
        exit(EXIT_FAILURE);
    } else if (status != 0) {
        printf("brk returned %d, unexpected result\n", status);
        exit(EXIT_FAILURE);
    }
}
