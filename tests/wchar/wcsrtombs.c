#include <assert.h>
#include <wchar.h>
#include <string.h>
#include <stdlib.h>

int main() {
    const wchar_t *wcs1 = L"";
    size_t mb_len1 = wcsrtombs(NULL, &wcs1, 0, NULL);
    assert(mb_len1 == 0);

    const wchar_t *wcs2 = L"Hello, world!";
    size_t mb_len2 = wcsrtombs(NULL, &wcs2, 0, NULL);
    assert(mb_len2 == strlen("Hello, world!"));

    // wcsrtombs resets wcs2 to zero, so we need to fill it again
    wcs2 = L"Hello, world!";
    char *mbs2 = malloc(mb_len2 + 1);
    wcsrtombs(mbs2, &wcs2, mb_len2 + 1, NULL);
    assert(strcmp(mbs2, "Hello, world!") == 0);
    free(mbs2);

    return 0;
}