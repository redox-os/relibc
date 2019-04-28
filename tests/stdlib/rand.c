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

    // Ensure rand_r() fails with NULL input
    if (rand_r(NULL) != EINVAL) {
        puts("rand_r(NULL) doesn't return EINVAL");
        exit(EXIT_FAILURE);
    }

    // Ensure rand_r() produces unique values
    int seed = 259;
    int rand_r_seed259_1 = rand_r((unsigned *)&seed);
    printf("%d\n", rand_r_seed259_1);

    int rand_r_seed259_2 = rand_r((unsigned *)&seed);
    printf("%d\n", rand_r_seed259_2);

    if (rand_r_seed259_1 == rand_r_seed259_2) {
        puts("rand_r() fails to produce unique values");
        exit(EXIT_FAILURE);
    }

    // Ensure rand_r() returns reproducible values
    seed = 259;
    int rand_r_seed259_1_2 = rand_r((unsigned *)&seed);
    printf("%d\n", rand_r_seed259_1_2);

    int rand_r_seed259_2_2 = rand_r((unsigned *)&seed);
    printf("%d\n", rand_r_seed259_2_2);

    if (rand_r_seed259_1 != rand_r_seed259_1_2
        || rand_r_seed259_2 != rand_r_seed259_2_2)
    {
        puts("rand_r() is not reproducible");
	exit(EXIT_FAILURE);
    }

    return 0;
}
