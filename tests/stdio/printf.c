#include <stdio.h>

int main(void) {
    int sofar = 0;
    int len = printf(
        "percent: %%\nstring: %s\nchar: %c\nchar: %c\nint: %d\n%nuint: %u\nhex: %x\nHEX: %X\nstring: %s\n",
        "String",
        'c',
        254,
        -16,
        &sofar,
        32,
        0xbeef,
        0xC0FFEE,
        "end"
    );
    printf("%%n returned %d, total len of write: %d\n", sofar, len);

    puts("\nPadding madness:");
    printf("% -5.3d %+3d\n", 1, 2);
    printf("%4c\n", 'c');
    printf("%#10.7x\n", 0xFF);
    printf("%#4.3o\n", 01);
    printf("%#x %#x\n", 0, 1);
    printf("%.0d %.0d\n", 0, 1);
    printf("(%05d) (%5d)\n", 123, 123);
    printf("(%05d) (%5d)\n", -123, -123);
    printf("(%13.0d)\n", 0);
    printf("%p\n", (void*) 0xABCDEF);
    printf("%p\n", (void*) 0);

    puts("\nPositional madness:");
    printf("%3$d %2$d %1$d\n", 2, 3, 4);
    printf("%.*3$d\n", 2, 0, 5);
    printf("|%-*6$.*5$s|%-*6$.*5$s|%*6$.*5$s|%*6$.*5$s|\n", "Fizz", "Buzz", "FizzBuzz", "TotalBuzz", 3, 8);
    printf("int: %*6$d double: %lf %lf %lf %lf\n", 5, 0.1, 0.2, 0.3, 0.4, 10);
    printf("%1$d %1$lf\n", 5, 0.1);
    printf("%1$d %lf\n", 5, 0.2);
    //printf("int: %*6$d no info on middle types\n", 5, 0.1, 0.2, 0.3, 0.4, 10);

    puts("\nFloat madness:");
    printf("%20e\n", 123.456789123);
    printf("%20E\n", 0.00001);
    printf("%20f\n", 123.456789123);
    printf("%20F\n", 0.00001);
    printf("%20e\n", -123.456789123);
    printf("%020e\n", -123.456789123);

    printf("%g\n", 100000.0);
    printf("%g\n", 1000000.0);
    printf("%e\n", 1000000.0);
    printf("%G\n", 0.0001);
    printf("%G\n", 0.00001);
    printf("%E\n", 0.00001);
}
