#include <unistd.h>
#include <stdio.h>

int main(void) {
    int status = brk((void*)100);
    printf("brk exited with status code %d\n", status);
    return 0;
}
