#include <stdio.h>
#include <wchar.h>

int main () {
    wchar_t *wcs = L"relibc";
    size_t len = wcslen(wcs);
    int width = wcswidth(wcs, len);
    printf("wcswidth(L\"%ls\", %d) = %d\n", wcs, len, width);
    return 0;
}
