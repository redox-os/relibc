#include <stdlib.h>
#include <stdio.h>

#include "test_helpers.h"

/* The output of these tests are checked against that from musl. Other
 * algorithms may yield different results and still comply with the
 * POSIX requirements. */

int main(void) {
    /* Should be enough to exercise the rollover branching in random()
     * for all possible state array sizes (up to 256 bytes, i.e. 64
     * 32-bit values). */
    size_t test_seq_len = 70;

    long random_result;

    // Should give same result as with seed 1
    puts("Uninitialized:");
    for (size_t i = 0; i < test_seq_len; i++) {
        random_result = random();
        printf("%ld\n", random_result);
    }

    puts("\nSeed 1:");
    srandom(1);
    for (size_t i = 0; i < test_seq_len; i++) {
        random_result = random();
        printf("%ld\n", random_result);
    }

    puts("\nSeed 1337:");
    srandom(1337);
    for (size_t i = 0; i < test_seq_len; i++) {
        random_result = random();
        printf("%ld\n", random_result);
    }

    /* 256 bytes (as below) is the largest possible amount of state
     * data. Created as a uint32_t to avoid possible alignment issues
     * with char. */
    uint32_t new_state[64] = {
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    };

    unsigned int seed = 42;

    // Should exercise the different branches in initstate()
    size_t sizes[] = {8, 31, 32, 63, 64, 127, 128, 255, 256};

    for (size_t j = 0; j < sizeof(sizes)/sizeof(size_t); j++) {
        size_t size = sizes[j];
        printf("\nSeed %d, size %ld:\n", seed, size);
        initstate(seed, (char *)new_state, size);

        for (size_t i = 0; i < test_seq_len; i++) {
            random_result = random();
            printf("%ld\n", random_result);
        }
    }

    /* Test that setstate() allows the use of a different state array,
     * and that it correctly returns the old value. */
    uint32_t other_new_state[64] = {
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    };

    initstate(seed, (char *)other_new_state, 32);
    printf("\nSeed %d, other state array:\n", seed);
    for (size_t i = 0; i < test_seq_len; i++) {
        random_result = random();
        printf("%ld\n", random_result);
    }

    char *should_be_other_new_state_ptr = setstate((char *)new_state);
    if (should_be_other_new_state_ptr == (char *)other_new_state) {
        puts("\nState data pointer restored correctly by setstate().");
    }
    else {
        puts("\nState data pointer NOT restored correctly by setstate().");
    }
    printf("\nSeed %d, back to first state array:\n", seed);
    for (size_t i = 0; i < test_seq_len; i++) {
        random_result = random();
        printf("%ld\n", random_result);
    }

    // Should yield NULL
    char *state_with_size_less_than_8 = initstate(seed, (char *)new_state, 7);
    printf("\nPointer returned by initstate with size < 8: %p\n",
        state_with_size_less_than_8);

    return 0;
}
