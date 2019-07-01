#include <stdlib.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    char * s = "azAZ9."; // test boundaries
    long l = a64l(s);
    if (l != 194301926) {
        printf("Invalid result: a64l(%s) = %ld\n", s, l);
        exit(EXIT_FAILURE);
    }
    printf("Correct a64l: %s = %ld\n", s, l);

    
    s = "azA"; // test null terminated string
    l = a64l(s);
    if (l != 53222) {
        printf("Invalid result: a64l(%s) = %ld\n", s, l);
        exit(EXIT_FAILURE);
    }
    printf("Correct a64l: %s = %ld\n", s, l);
    
    /* Test near boundaries of digit character mapping, and near
     * boundaries for number of digits */
    long l64a_test_values[] = {0, 1, 2, 11, 12, 37, \
        38, 63, \
        64, 65, 4095, \
        4096, 262143, \
        262144, 16777215, \
        16777216, 1073741823, \
        1073741824, 2147483647};
    
    // l64a tests
    for (size_t i = 0; i < sizeof(l64a_test_values)/sizeof(long); i++) {
        printf("l64a(%ld): %s\n", l64a_test_values[i], l64a(l64a_test_values[i]));
    }
    
    // a64l(l64a(x)) round-trip tests
    for (size_t i = 0; i < sizeof(l64a_test_values)/sizeof(long); i++) {
        printf("a64l(l64a(%ld)): %ld\n", l64a_test_values[i], a64l(l64a(l64a_test_values[i])));
    }
    
    /* For testing 32-bit truncation behavior (for platforms where long
     * is larger than 32 bits). Note that the behavior for a64l() and
     * l64a() is unspecified for negative values. */
    int64_t test_value_64bit = 0x7edcba9876543210;
    printf("l64a(x) (lower 32 bits of x are %ld): %s\n", ((long)test_value_64bit) & 0xffffffff, l64a((long)test_value_64bit));
    
    /* Test for trunctation in l64a(a64(x)) round trip (POSIX says the
     * result of that is "x in the low-order 32-bits". */
    printf("a64l(l64a(x)) (lower 32 bits of x are %ld): %ld\n", ((long)test_value_64bit) & 0xffffffff, a64l(l64a((long)test_value_64bit)));
}
