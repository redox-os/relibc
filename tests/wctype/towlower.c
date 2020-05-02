#include <stdio.h>
#include <wchar.h>
#include <wctype.h>

int main() {
    wchar_t *str = L"HaLf WiDe ChAr StRiNg!\n";
    fputws(str, stdout);
    for (int i = 0; i < wcslen(str); i++) {
        putwchar(towlower(str[i]));
    }
    return 0;
}