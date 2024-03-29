#include <stddef.h>
#include <stdlib.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    char* endptr = 0;
    double d;

    char* inputs[] = {
        "a 1 hello", " 1 hello", "1 hello 2",
        "10.123", "010.123", "-5.3",
        "0x10.123", "0x1.23", "0x3.21",

        "1e5", "1e+5", "1e-5",
        "1e5   ", "1e+5   ", "1e-5   ",

        "1e10", "1eXXXX", "1e", "1e ",
        "1e+10", "1e+XXXX", "1e+", "1e+ ",
        "1e-10", "1e-XXXX", "1e-", "1e- ",

        "-1e5", "-1e+5", "-1e-5",
        "-1e5   ", "-1e+5   ", "-1e-5   ",

        "-1e10", "-1eXXXX", "-1e", "-1e ",
        "-1e+10", "-1e+XXXX", "-1e+", "-1e+ ",
        "-1e-10", "-1e-XXXX", "-1e-", "-1e- ",

        "12.34e5", "12.34e+5", "12.34e-5",
        "12.34e5   ", "12.34e+5   ", "12.34e-5   ",

        "12.34e10", "12.34eXXXX", "12.34e", "12.34e ",
        "12.34e+10", "12.34e+XXXX", "12.34e+", "12.34e+ ",
        "12.34e-10", "12.34e-XXXX", "12.34e-", "12.34e- ",

        "-12.34e5", "-12.34e+5", "-12.34e-5",
        "-12.34e5   ", "-12.34e+5   ", "-12.34e-5   ",

        "-12.34e10", "-12.34eXXXX", "-12.34e", "-12.34e ",
        "-12.34e+10", "-12.34e+XXXX", "-12.34e+", "-12.34e+ ",
        "-12.34e-10", "-12.34e-XXXX", "-12.34e-", "-12.34e- ",

        "0x0.3p10", "-0x0.3p10", "0x0.3p-5", "-0x0.3p-5",
        "0x1.4p3", "0x1.4p-3", "-0x1.4p3", "-0x1.4p-3",
        "0x10.1p0", "0x10.1p-0", "-0x10.1p0", "-0x10.1p-0",

        "0.5e0", "0.5e1", "0.5e2", "0.5e3", "0.5e4",
        "0.5e5", "0.5e6", "0.5e7", "0.5e8", "0.5e9",
        "0.5e10", "0.5e11", "0.5e12", "0.5e13", "0.5e14",
        "0.5e15", "0.5e16", "0.5e17", "0.5e18", "0.5e19",
        "0.5e20", "0.5e21", "0.5e22", "0.5e23", "0.5e24",
        "0.5e25", "0.5e26", "0.5e27", "0.5e28", "0.5e29",
        "0.5e30", "0.5e31", "0.5e32", "0.5e33", "0.5e34",
        "0.5e35", "0.5e36", "0.5e37", "0.5e38",

        "-0.5e0", "-0.5e1", "-0.5e2", "-0.5e3", "-0.5e4",
        "-0.5e5", "-0.5e6", "-0.5e7", "-0.5e8", "-0.5e9",
        "-0.5e10", "-0.5e11", "-0.5e12", "-0.5e13", "-0.5e14",
        "-0.5e15", "-0.5e16", "-0.5e17", "-0.5e18", "-0.5e19",
        "-0.5e20", "-0.5e21", "-0.5e22", "-0.5e23", "-0.5e24",
        "-0.5e25", "-0.5e26", "-0.5e27", "-0.5e28", "-0.5e29",
        "-0.5e30", "-0.5e31", "-0.5e32", "-0.5e33", "-0.5e34",
        "-0.5e35", "-0.5e36", "-0.5e37", "-0.5e38",

        "-0",

        "INF", "inf", "iNf", "Inf foobarbaz",
        "+INF", "+inf", "+iNf", "+Inf foobarbaz",
        "-INF", "-inf", "-iNf", "-Inf foobarbaz",

        "NaN0.1e5", "nan-37", "nAn1.05", "Nan foo bar baz",
        "+NaN0.1e5", "+nan-37", "+nAn1.05", "+Nan foo bar baz",
        "-NaN0.1e5", "-nan-37", "-nAn1.05", "-Nan foo bar baz",

    };
    for (size_t i = 0; i < sizeof(inputs) / sizeof(char*); i += 1) {
        d = strtod(inputs[i], &endptr);
        printf("d: %f Endptr: \"%s\"\n", d, endptr);
    }
}
