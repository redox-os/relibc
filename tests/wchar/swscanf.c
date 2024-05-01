/* swscanf example */
#include <wchar.h>
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


void test(wchar_t* fmt_in, wchar_t* input, struct params *p, ...) {
    va_list args;
    va_start(args, p);
    wint_t ret = vswscanf(input, fmt_in, args);
    va_end(args);

    wprintf(
        L"%d, { sa: %hhd, ia: %d, ib: %d, ic: %d, fa: %f, da: %lf, ptr: %p, char: %c, string1: %s, string2: %s, string3: %s, string4: %s }\n",
        ret, p->sa, p->ia, p->ib, p->ic, p->fa, p->da, p->ptr, p->c, p->string1, p->string2, p->string3, p->string4
    );
}

int main ()
{
    struct params p = { .c = 'a' };

    test(L"%hd %d", L"12 345", &p, &p.sa, &p.ia);
    test(L"%x %i %i", L"12 0x345 010", &p, &p.ia, &p.ib, &p.ic);
    test(L"%f.%lf", L"0.1.0.2", &p, &p.fa, &p.da);
    test(L"%p", L"0xABCDEF", &p, &p.ptr);
    test(L"%s", L"Hello World", &p, &p.string1);
    test(L"%3i", L"0xFF", &p, &p.ia);
    test(L"%c%3c", L"hello", &p, &p.c, &p.string1);
    test(L"test: %2i%n", L"test: 0xFF", &p, &p.ia, &p.ib);
    test(L"hello world%%", L"hello world%", &p);
    test(L"h%1[ae]ll%1[^a] wor%1[^\n]%[d]", L"hello world", &p, &p.string1, &p.string2, &p.string3, &p.string4);
    test(L"h%1[ae]ll%1[^a] wor%1[^\n]%[d]", L"halle worfdddddd", &p, &p.string1, &p.string2, &p.string3, &p.string4);
    test(L"h%1[ae]ll%1[^a] wor%1[^\n]%[d]", L"halle worfdddddd", &p, &p.string1, &p.string2, &p.string3, &p.string4);
    test(L"%[^a]%[b]", L"testbbbb", &p, &p.string1, &p.string2);

    // Scanf stolen from the url parsing in curl
    wchar_t protobuf[16];
    wchar_t slashbuf[4];
    wchar_t hostbuf[100];
    wchar_t pathbuf[100];

    // don't push NUL, make sure scanf does that
    memset(protobuf, 97, 16);
    memset(slashbuf, 97, 4);
    memset(hostbuf, 97, 100);
    memset(pathbuf, 97, 100);

    int ret = swscanf(
        L"https://redox-os.org\0# extra garbage for nul test", L"%15[^\n/:]:%3[/]%[^\n/?#]%[^\n]",
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

    wprintf(L"%d \"%s\" \"%s\" \"%s\" \"%s\"\n", ret, &protobuf, &slashbuf, &hostbuf, &pathbuf);


        wchar_t str1[20];
    wchar_t str2[20];
    wint_t status = swscanf(L"Привет мир", L"%ls %ls", str1, str2);
    wprintf(L"--> 1 %d \"%ls\" \"%ls\"\n", status, &str1, &str2);
wchar_t str12[20];
    wchar_t str22[20];
    wint_t status2 = swscanf(L"Привет мир", L"%s %s", str12, str22);
    wprintf(L"==> 2 %d \"%s\" \"%s\"\n", status2, &str12, &str22);



    wchar_t wcs[4];
    wint_t status3 = swscanf(L"zß水🍌", L"%ls", &wcs);
    wprintf(L"==> 2 %d \"%ls\"\n", status3, &wcs);

    // It should be %ls
    // wint_t status = swscanf(L"Привет мир", L"l%s %ls", str1, str2);
    // wprintf(L"%d \"%ls\" \"%ls\"\n", status, &str1, &str2);

  return 0;
}
