#include <stdio.h>
#include <stddef.h>
#include <stdlib.h>

#define ARRAY_SIZE(x) (sizeof(x)/sizeof(x[0]))

int values[] = { 23, 16, 8, 4, 42, 15 };

int cmpfunc (const void * a_ptr, const void * b_ptr) {
    int a = *(const int *)a_ptr;
    int b = *(const int *)b_ptr;
    return a - b;
}

int main () {
    size_t i;

    printf("Before: ");
    for(i = 0; i < ARRAY_SIZE(values); i++) {
        printf("%d ", values[i]);
    }
    printf("\n");

    qsort(values, ARRAY_SIZE(values), sizeof(int), cmpfunc);

    printf("After: ");
    for(i = 0; i < ARRAY_SIZE(values); i++) {
        printf("%d ", values[i]);
    }
    printf("\n");

    return 0;
}
