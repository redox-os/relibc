#include <string.h>
#include <stdio.h>
#include <wchar.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    wchar_t *wcs = L"zß水🍌";

    int i;
    for (i = 0; wcs[i] != L'\0'; i++)
    {
        if (0xFFFFFFFFu == putwchar(wcs[i]))
        {
            printf("Unable to putwchar() the wide character.\n");
            exit(EXIT_FAILURE);
        }
    }
}
