#include <stdlib.h>
#include <unistd.h>

int main(int argc, char **argv) {
    write(STDERR_FILENO, "malloc\n", 7);
    char * ptr = (char *)malloc(256);
    write(STDERR_FILENO, "set\n", 4);
    int i;
    for(i = 0; i < 256; i++) {
        ptr[i] = (char)i;
    }
    write(STDERR_FILENO, "free\n", 5);
    free(ptr);
}
