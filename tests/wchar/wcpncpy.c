#include <wchar.h>
#include <assert.h>
#include <string.h>

int main() {
    wchar_t src[] = L"Привет мир";
    wchar_t dest[10];
    wchar_t* result = wcpncpy(dest, src, 5);

    assert(wcsncmp(dest, src, 5) == 0);
    assert(*result == L'\0');
    assert(result == dest + 5);

    return 0;
}
