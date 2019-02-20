#include <stdarg.h>
#include <stdio.h>

struct params {
    short sa;
    int ia;
    int ib;
    int ic;
    float fa;
    double da;
    int *ptr;
    char c;
    char string[20];
};

void test(char* fmt_in, char* input, struct params *p, ...) {
    va_list args;
    va_start(args, p);
    int ret = vsscanf(input, fmt_in, args);
    va_end(args);

    printf(
        "%d, { sa: %hhd, ia: %d, ib: %d, ic: %d, fa: %f, da: %lf, ptr: %p, char: %c, string: %s }\n",
        ret, p->sa, p->ia, p->ib, p->ic, p->fa, p->da, p->ptr, p->c, p->string
    );
}

int main(void) {
    struct params p = { .c = 'a' };

    test("%hhd %d", "12 345", &p, &p.sa, &p.ia);
    test("%x %i %i", "12 0x345 010", &p, &p.ia, &p.ib, &p.ic);
    test("%f.%lf", "0.1.0.2", &p, &p.fa, &p.da);
    test("%p", "0xABCDEF", &p, &p.ptr);
    test("%s", "Hello World", &p, &p.string);
    test("%3i", "0xFF", &p, &p.ia);
    test("%c%3c", "hello", &p, &p.c, &p.string);
    test("test: %2i%n", "test: 0xFF", &p, &p.ia, &p.ib);
    test("hello world%%", "hello world%", &p);
}
