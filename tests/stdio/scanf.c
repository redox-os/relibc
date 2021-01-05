#include <stdarg.h>
#include <stdio.h>

#include "test_helpers.h"

struct params {
    short sa;
    int ia;
    int ib;
    int ic;
    float fa;
    double da;
    int *ptr;
    char c;
    char string1[20];
    char string2[20];
    char string3[20];
    char string4[20];
};

void test(char* fmt_in, char* input, struct params *p, ...) {
    va_list args;
    va_start(args, p);
    int ret = vsscanf(input, fmt_in, args);
    va_end(args);

    printf(
        "%d, { sa: %hhd, ia: %d, ib: %d, ic: %d, fa: %f, da: %lf, ptr: %p, char: %c, string1: %s, string2: %s, string3: %s, string4: %s }\n",
        ret, p->sa, p->ia, p->ib, p->ic, p->fa, p->da, p->ptr, p->c, p->string1, p->string2, p->string3, p->string4
    );
}

int main(void) {
    struct params p = { .c = 'a' };

    test("%hd %d", "12 345", &p, &p.sa, &p.ia);
    test("%x %i %i", "12 0x345 010", &p, &p.ia, &p.ib, &p.ic);
    test("%f.%lf", "0.1.0.2", &p, &p.fa, &p.da);
    test("%p", "0xABCDEF", &p, &p.ptr);
    test("%s", "Hello World", &p, &p.string1);
    test("%3i", "0xFF", &p, &p.ia);
    test("%c%3c", "hello", &p, &p.c, &p.string1);
    test("test: %2i%n", "test: 0xFF", &p, &p.ia, &p.ib);
    test("hello world%%", "hello world%", &p);
    test("h%1[ae]ll%1[^a] wor%1[^\n]%[d]", "hello world", &p, &p.string1, &p.string2, &p.string3, &p.string4);
    test("h%1[ae]ll%1[^a] wor%1[^\n]%[d]", "halle worfdddddd", &p, &p.string1, &p.string2, &p.string3, &p.string4);
    test("h%1[ae]ll%1[^a] wor%1[^\n]%[d]", "halle worfdddddd", &p, &p.string1, &p.string2, &p.string3, &p.string4);
    test("%[^a]%[b]", "testbbbb", &p, &p.string1, &p.string2);


    // Scanf stolen from the url parsing in curl
    char protobuf[16];
    char slashbuf[4];
    char hostbuf[100];
    char pathbuf[100];

    // don't push NUL, make sure scanf does that
    memset(protobuf, 97, 16);
    memset(slashbuf, 97, 4);
    memset(hostbuf, 97, 100);
    memset(pathbuf, 97, 100);

    int ret = sscanf(
        "https://redox-os.org\0# extra garbage for nul test", "%15[^\n/:]:%3[/]%[^\n/?#]%[^\n]",
        &protobuf, &slashbuf, &hostbuf, &pathbuf
    );
    if (ret < 4) {
        *pathbuf = 0;
    }
    if (ret < 3) {
        *hostbuf = 0;
    }
    if (ret < 2) {
        *slashbuf = 0;
    }
    if (ret < 1) {
        *protobuf = 0;
    }

    printf("%d \"%s\" \"%s\" \"%s\" \"%s\"\n", ret, &protobuf, &slashbuf, &hostbuf, &pathbuf);
}
