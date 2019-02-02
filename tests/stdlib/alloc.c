#include <malloc.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h> /* for SIZE_MAX */

int main(int argc, char ** argv) {
    char * ptr = (char *)malloc(256);
    printf("malloc %p\n", ptr);
    int i;
    for(i = 0; i < 256; i++) {
        ptr[i] = (char)i;
    }
    free(ptr);

    char * ptrc = (char *)calloc(256, 1);
    printf("calloc %p\n", ptrc);
    for(i = 0; i < 256; i++) {
        ptrc[i] = (char)i;
    }
    free(ptrc);

    char * ptrco = (char *)calloc(SIZE_MAX, SIZE_MAX);
    printf("calloc (overflowing) %p\n", ptrco);
    free(ptrco); /* clean up correctly even if overflow is not handled */

    char * ptra = (char *)memalign(256, 256);
    printf("memalign %p\n", ptra);
    for(i = 0; i < 256; i++) {
        ptra[i] = (char)i;
    }
    free(ptra);

    return 0;
}
