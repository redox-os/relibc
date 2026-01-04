#include <stdio.h>
#include <stdlib.h>
#include <malloc.h>
#include <string.h>
#include <assert.h>

int main(void) {
    size_t request_size = 27;
    char *ptr = (char *)malloc(request_size);

    if (!ptr) {
        fprintf(stderr, "Allocation failed\n");
        return 1;
    }

    size_t actual_size = malloc_usable_size(ptr);

    printf("malloc: %zu bytes\n", request_size);
    printf("malloc_usable_size: %zu bytes\n", actual_size);

    assert(actual_size >= request_size);
    memset(ptr, 'A', actual_size); 
    ptr[actual_size - 1] = '\0';

    assert(ptr[0] == 'A');
    assert(ptr[actual_size - 2] == 'A');
    assert(ptr[actual_size - 1] == '\0');
    free(ptr);

    return 0;
}
