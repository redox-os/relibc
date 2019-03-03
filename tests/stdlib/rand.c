#include <stdlib.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    // Uninitialized generator
    int rand_uninit = rand();
    printf("%d\n", rand_uninit);

    // Testing the reproducibility of values
    srand(259);
    int rand_seed259_1 = rand();
    printf("%d\n", rand_seed259_1);

    srand(259);
    int rand_seed259_2 = rand();
    printf("%d\n", rand_seed259_2);

    if (rand_seed259_1 != rand_seed259_2) {
        puts("rand() doesn't return reproducible values");
        exit(EXIT_FAILURE);
    }

    // Seed value 1 should return the same values as the ininitialized generator
    srand(1);
    int rand_seed1 = rand();
    printf("%d\n", rand_seed1);

    if (rand_uninit != rand_seed1) {
        puts("srand(1) doesn't work");
        exit(EXIT_FAILURE);
    }
}
