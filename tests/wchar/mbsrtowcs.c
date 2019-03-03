#include <stdio.h>
#include <string.h>
#include <wchar.h>

#include "test_helpers.h"

void print_as_wide(const char* mbstr)
{
    mbstate_t state;
    memset(&state, 0, sizeof state);
    size_t len = 1 + mbsrtowcs(NULL, &mbstr, 0, &state);
    wchar_t wstr[len];
    mbsrtowcs(&wstr[0], &mbstr, len, &state);

    //Should be 5
    printf("The length, including '\\0': %li \n",len);

    //missing wprintf to print this wide string
    //wprintf(L"The wide string: %ls \n", &wstr[0]);
}

int main(void) {
    const char* mbstr = u8"z\u00df\u6c34\U0001f34c"; // or u8"zß水🍌"
    print_as_wide(mbstr);
}
