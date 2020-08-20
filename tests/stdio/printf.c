#include <stdio.h>
#include <stdlib.h> // free()
#include <math.h> // INFINITY, NAN constants

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
    printf("%%.5g: %.5g\n", -123.456789123);
    printf("%%.5f: %.5f\n", -123.456789123);
    printf("%%.5e: %.5e\n", -123.456789123);
    printf("%%.*g: %.*g\n", 2, -123.456789123);
    printf("%%.*f: %.*f\n", 2, -123.456789123);
    printf("%%.*e: %.*e\n", 2, -123.456789123);
    printf("%%.*2$g: %.*2$g\n", -123.456789123, 5);
    printf("%%.*2$f: %.*2$f\n", -123.456789123, 5);
    printf("%%.*2$e: %.*2$e\n", -123.456789123, 5);

    printf("%g\n", 100000.0);
    printf("%g\n", 1000000.0);
    printf("%e\n", 1000000.0);
    printf("%G\n", 0.0001);
    printf("%G\n", 0.00001);
    printf("%E\n", 0.00001);

    double nonfinites[] = {INFINITY, -INFINITY, NAN, -NAN};
    char *float_formats[] = {"%e", "%E", "%f", "%F", "%g", "%G"};
    puts("\nNon-finite float madness:");
    for (size_t i = 0; i < sizeof(float_formats)/sizeof(char *); i++) {
        printf("%s:", float_formats[i]);
        for (size_t j = 0; j < sizeof(nonfinites)/sizeof(double); j++) {
            printf(" ");
            printf(float_formats[i], nonfinites[j]);
        }
        printf("\n");
    }

    puts("Things that have been buggy");
    printf("%s%0*lu\n", "+", 2, 5l); // this format string was found in GDB

    puts("Testing asprintf...");
    char *s = NULL;
    int res = asprintf(&s, "test string");
    printf("printed: %s, value: %d\n", s, res);
    free(s);
    res = asprintf(&s, "test string %d", 2);
    printf("printed: %s, value: %d\n", s, res);
    free(s);
    res = asprintf(&s, "test %s %d", "string", 2);
    printf("printed: %s, value: %d\n", s, res);
    free(s);
}
