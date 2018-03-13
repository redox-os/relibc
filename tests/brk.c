#include <unistd.h>
#include <stdio.h>

int main(int argc, char** argv) {
    int status = brk((void*)100);
    printf("brk exited with status code %d\n", status);
    return 0;
}
