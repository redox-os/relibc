#include <iso646.h>
#include <stdint.h>
#include <stdio.h>

#include "test_helpers.h"

int main() {
    uint8_t a_bits = 0xa; // 0b0000_1010
    uint8_t b_bits = 0xc; // 0b0000_1100

    uint8_t c_bits;
    int c_bool;

    printf("0 and 0: %d\n", 0 and 0);
    printf("1 and 0: %d\n", 1 and 0);
    printf("0 and 1: %d\n", 0 and 1);
    printf("1 and 1: %d\n", 1 and 1);

    c_bool = 0;
    c_bool and_eq 0;
    printf("0 and_eq 0: %d\n", c_bool);
    c_bool = 1;
    c_bool and_eq 0;
    printf("1 and_eq 0: %d\n", c_bool);
    c_bool = 0;
    c_bool and_eq 1;
    printf("0 and_eq 1: %d\n", c_bool);
    c_bool = 1;
    c_bool and_eq 1;
    printf("1 and_eq 1: %d\n", c_bool);

    c_bits = a_bits bitand b_bits;
    printf("a bitand b: %d\n", c_bits);

    c_bits = a_bits bitor b_bits;
    printf("a bitor b: %d\n", c_bits);

    c_bits = compl a_bits;
    printf("compl a: %d\n", c_bits);

    printf("not 0: %d\n", not 0);
    printf("not 1: %d\n", not 1);

    c_bool = 0;
    c_bool not_eq 0;
    printf("0 not_eq 0: %d\n", c_bool);
    c_bool = 1;
    c_bool not_eq 0;
    printf("1 not_eq 0: %d\n", c_bool);
    c_bool = 0;
    c_bool not_eq 1;
    printf("0 not_eq 1: %d\n", c_bool);
    c_bool = 1;
    c_bool not_eq 1;
    printf("1 not_eq 1: %d\n", c_bool);

    printf("0 or 0: %d\n", 0 or 0);
    printf("1 or 0: %d\n", 1 or 0);
    printf("0 or 1: %d\n", 0 or 1);
    printf("1 or 1: %d\n", 1 or 1);

    c_bool = 0;
    c_bool or_eq 0;
    printf("0 or_eq 0: %d\n", c_bool);
    c_bool = 1;
    c_bool or_eq 0;
    printf("1 or_eq 0: %d\n", c_bool);
    c_bool = 0;
    c_bool or_eq 1;
    printf("0 or_eq 1: %d\n", c_bool);
    c_bool = 1;
    c_bool or_eq 1;
    printf("1 or_eq 1: %d\n", c_bool);

    printf("0 xor 0: %d\n", 0 xor 0);
    printf("1 xor 0: %d\n", 1 xor 0);
    printf("0 xor 1: %d\n", 0 xor 1);
    printf("1 xor 1: %d\n", 1 xor 1);

    c_bool = 0;
    c_bool xor_eq 0;
    printf("0 xor_eq 0: %d\n", c_bool);
    c_bool = 1;
    c_bool xor_eq 0;
    printf("1 xor_eq 0: %d\n", c_bool);
    c_bool = 0;
    c_bool xor_eq 1;
    printf("0 xor_eq 1: %d\n", c_bool);
    c_bool = 1;
    c_bool xor_eq 1;
    printf("1 xor_eq 1: %d\n", c_bool);
}
