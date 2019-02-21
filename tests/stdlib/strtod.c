#include <stdlib.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    char* endptr = 0;
    double d;

    char* inputs[] = {
        "a 1 hello", " 1 hello", "1 hello 2",
        "10.123", "010.123", "-5.3",
        "0x10.123", "0x1.23", "0x3.21"
    };
    for (int i = 0; i < sizeof(inputs) / sizeof(char*); i += 1) {
        d = strtod(inputs[i], &endptr);
        printf("d: %f Endptr: \"%s\"\n", d, endptr);
    }
}
