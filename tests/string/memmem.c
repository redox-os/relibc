#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "test_helpers.h"

int main(void) {
    uint8_t haystack[] = {1, 1, 2, 1, 2, 3, 1, 2, 3, 4};
    uint8_t present_needle[] = {1, 2, 3};
    uint8_t absent_needle[] = {1, 2, 3, 4, 5};
    uint8_t long_needle[] = {1, 1, 2, 1, 2, 3, 1, 2, 3, 4, 1};

    size_t haystacklen = sizeof(haystack);
    size_t present_needlelen = sizeof(present_needle);
    size_t absent_needlelen = sizeof(absent_needle);
    size_t long_needlelen = sizeof(long_needle);

    uint8_t *present_needle_match_ptr = memmem(haystack, haystacklen, present_needle, present_needlelen);
    assert(present_needle_match_ptr == haystack + 3);

    uint8_t *absent_needle_match_ptr = memmem(haystack, haystacklen, absent_needle, absent_needlelen);
    assert(absent_needle_match_ptr == NULL);

    // Explicitly specified to return haystack for needlelen == 0.
    uint8_t *zero_needle_match_ptr = memmem(haystack, haystacklen, present_needle, 0);
    assert(zero_needle_match_ptr == haystack);

    uint8_t *long_needle_match_ptr = memmem(haystack, haystacklen, long_needle, long_needlelen);
    assert(long_needle_match_ptr == NULL);
}
