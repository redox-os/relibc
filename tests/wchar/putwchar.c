#include <string.h>
#include <stdio.h>
#include <wchar.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    wchar_t *wcs = L"zß水🍌";

    for (int i = 0; wcs[i] != L'\0'; i++) {
        wint_t status = putwchar(wcs[i]);
        ERROR_IF(putwchar, status, == WEOF);
    }
}
