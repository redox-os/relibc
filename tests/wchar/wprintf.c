#include <stdio.h>
#include <stdlib.h> // free()
#include <wchar.h>
#include <math.h> // INFINITY, NAN constants

int main(void) {
    int sofar = 0;
    int len = wprintf(
        L"percent: %%\nwstring: %ls\nchar: %lc\nint: %d\n%nuint: %u\nhex: %x\nHEX: %X\nstring: %s\n",
        L"String",
        L'c',
        -16,
        &sofar,
        32,
        0xbeef,
        0xC0FFEE,
        "end"
    );
    wprintf(L"%%n returned %d, total len of write: %d\n", sofar, len);

    wprintf(L"\nPadding madness:\n");
    wprintf(L"% -5.3d %+3d\n", 1, 2);
    wprintf(L"%4c\n", 'c');
    wprintf(L"%#10.7x\n", 0xFF);
    wprintf(L"%#4.3o\n", 01);
    wprintf(L"%#x %#x\n", 0, 1);
    wprintf(L"%.0d %.0d\n", 0, 1);
    wprintf(L"(%05d) (%5d)\n", 123, 123);
    wprintf(L"(%05d) (%5d)\n", -123, -123);
    wprintf(L"(%13.0d)\n", 0);
    wprintf(L"%p\n", (void*) 0xABCDEF);
    wprintf(L"%p\n", (void*) 0);

    wprintf(L"\nPositional madness:\n");
    wprintf(L"%3$d %2$d %1$d\n", 2, 3, 4);
    wprintf(L"%.*3$d\n", 2, 0, 5);
    wprintf(L"|%-*6$.*5$s|%-*6$.*5$s|%*6$.*5$s|%*6$.*5$s|\n", "Fizz", "Buzz", "FizzBuzz", "TotalBuzz", 3, 8);
    wprintf(L"int: %*6$d double: %lf %lf %lf %lf\n", 5, 0.1, 0.2, 0.3, 0.4, 10);
    wprintf(L"%1$d %1$lf\n", 5, 0.1);
    wprintf(L"%1$d %lf\n", 5, 0.2);
    //wprintf(L"int: %*6$d no info on middle types\n", 5, 0.1, 0.2, 0.3, 0.4, 10);

    wprintf(L"\nFloat madness:\n");
    wprintf(L"%20e\n", 123.456789123);
    wprintf(L"%20E\n", 0.00001);
    wprintf(L"%20f\n", 123.456789123);
    wprintf(L"%20F\n", 0.00001);
    wprintf(L"%20e\n", -123.456789123);
    wprintf(L"%020e\n", -123.456789123);
    wprintf(L"%%.5g: %.5g\n", -123.456789123);
    wprintf(L"%%.5f: %.5f\n", -123.456789123);
    wprintf(L"%%.5e: %.5e\n", -123.456789123);
    wprintf(L"%%.*g: %.*g\n", 2, -123.456789123);
    wprintf(L"%%.*f: %.*f\n", 2, -123.456789123);
    wprintf(L"%%.*e: %.*e\n", 2, -123.456789123);
    wprintf(L"%%.*2$g: %.*2$g\n", -123.456789123, 5);
    wprintf(L"%%.*2$f: %.*2$f\n", -123.456789123, 5);
    wprintf(L"%%.*2$e: %.*2$e\n", -123.456789123, 5);

    wprintf(L"%g\n", 100000.0);
    wprintf(L"%g\n", 1000000.0);
    wprintf(L"%e\n", 1000000.0);
    wprintf(L"%G\n", 0.0001);
    wprintf(L"%G\n", 0.00001);
    wprintf(L"%E\n", 0.00001);

    double nonfinites[] = {INFINITY, -INFINITY, NAN, -NAN};
    wchar_t *float_formats[] = {L"%e", L"%E", L"%f", L"%F", L"%g", L"%G"};
    wprintf(L"\nNon-finite float madness:\n");
    for (size_t i = 0; i < sizeof(float_formats)/sizeof(char *); i++) {
        wprintf(L"%ls:", float_formats[i]);
        for (size_t j = 0; j < sizeof(nonfinites)/sizeof(double); j++) {
            wprintf(L" ");
            wprintf(float_formats[i], nonfinites[j]);
        }
        wprintf(L"\n");
    }

    wprintf(L"Things that have been buggy\n");
    wprintf(L"%s%0*lu\n", "+", 2, 5l); // this format string was found in GDB

    wprintf(L"Testing asprintf...\n");
    char *s = NULL;
    int res = asprintf(&s, "test string");
    wprintf(L"printed: %s, value: %d\n", s, res);
    free(s);
    res = asprintf(&s, "test string %d", 2);
    wprintf(L"printed: %s, value: %d\n", s, res);
    free(s);
    res = asprintf(&s, "test %s %d", "string", 2);
    wprintf(L"printed: %s, value: %d\n", s, res);
    free(s);
}
