#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "test_helpers.h"

#define MAX(x, y) (((x) > (y)) ? (x) : (y))
#define MIN(x, y) (((x) < (y)) ? (x) : (y))

int main(void) {
    const uint8_t UNTOUCHED_BYTE = 0x00;
    const uint8_t TOUCHED_BYTE = 0xff;
    // In order to fully exercise the implementation, this should be at least 3 times the largest possible chunk size
    const size_t BUFFER_LEN = 64;

    uint8_t *s1_buffer = malloc(BUFFER_LEN);
    uint8_t *s2_buffer = malloc(BUFFER_LEN);

    // Loop through all possible combinations of s1 and s2 alignments and slice length within the buffers allocated
    for (size_t s1_offset = 0; s1_offset < BUFFER_LEN; s1_offset++) {
        for (size_t s2_offset = 0; s2_offset < BUFFER_LEN; s2_offset++) {
            size_t n_max = BUFFER_LEN - MAX(s1_offset, s2_offset);
            for (size_t n = 1; n <= n_max; n++) {
                // Clear buffers
                memset(s1_buffer, UNTOUCHED_BYTE, BUFFER_LEN);
                memset(s2_buffer, UNTOUCHED_BYTE, BUFFER_LEN);

                // Fill s2 subslice
                memset(s2_buffer + s2_offset, TOUCHED_BYTE, n);

                // Do the actual memcpy of the slice of interest
                memcpy(s1_buffer + s1_offset, s2_buffer + s2_offset, n);

                // Check that area below the slice of interest is untouched
                for (size_t i = 0; i < s1_offset; i++) {
                    assert(s1_buffer[i] == UNTOUCHED_BYTE);
                }

                // Check that the slice of interest was copied
                assert(memcmp(s1_buffer + s1_offset, s2_buffer + s2_offset, n) == 0);

                // Check that area above the slice of interest is untouched
                for (size_t i = s1_offset + n; i < BUFFER_LEN; i++) {
                    assert(s1_buffer[i] == UNTOUCHED_BYTE);
                }
            }
        }
    }

    free(s1_buffer);
    free(s2_buffer);
}