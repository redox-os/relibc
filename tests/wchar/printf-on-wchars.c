#include <stdio.h>
#include <sys/types.h>
#include <wchar.h>

int main() {
    wint_t a = L'1';
    wint_t b = L'2';
    wint_t c = L'a';
    wint_t d = L'b';
    printf("This is a few one-byte chars: %lc %lc %lc %lc\n", a, b, c, d);
    wchar_t *s = L"Hello World";
    printf("Long one-byte string: %ls\n", s);

    a = L'❤';
    b = L'R';
    c = L'😠';
    d = L'C';
    printf("This is a few multi-byte chars: %lc %lc %lc %lc\n", a, b, c, d);

    s = L"👉😎👉 Zoop!";
    printf("Long multi-byte string: %ls\n", s);
}
