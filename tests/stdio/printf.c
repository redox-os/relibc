#include <stdio.h>

int main(int argc, char ** argv) {
    int len = printf(
        "percent: %%\nstring: %s\nchar: %c\nchar: %c\nint: %d\nuint: %u\nhex: %x\nHEX: %X\nstring: %s\n",
        "String",
        'c',
        254,
        -16,
        32,
        0xbeef,
        0xC0FFEE,
        "end"
    );
    printf("len of previous write: %d\n", len);
    return 0;
}
