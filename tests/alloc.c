#include <stdio.h>
#include <stdlib.h>

int main(int argc, char ** argv) {
    char * ptr = (char *)malloc(256);
    printf("malloc %p\n", ptr);
    int i;
    for(i = 0; i < 256; i++) {
        ptr[i] = (char)i;
    }
    free(ptr);

    char * ptrc = (char *)calloc(256,1);
    printf("calloc %p\n", ptrc);
    for(i = 0; i < 256; i++) {
        ptrc[i] = (char)i;
    }
    free(ptrc);

}
