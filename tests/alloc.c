#include <stdlib.h>

int main(int argc, char **argv) {
    char * ptr = (char *)malloc(256);
    int i;
    for(i = 0; i < 256; i++) {
        ptr[i] = (char)i;
    }
    free(ptr);
}
