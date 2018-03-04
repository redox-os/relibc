#include <stdio.h>

int main(int argc, char ** argv) {
    printf(
        "percent: %%\nstring: %s\nchar: %c\nint: %d\nuint: %u\nhex: %x\nHEX: %X\nstring: %s\n",
        "String",
        'c',
        -16,
        32,
        0xbeef,
        0xC0FFEE,
        "end"
    );
    return 0;
}
