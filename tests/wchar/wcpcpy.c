#include <wchar.h>
#include <assert.h>
#include <string.h>

int main() {
    wchar_t src[] = L"こんにちは世界!";
    wchar_t dest[9];

    wchar_t* result = wcpcpy(dest, src);

    assert(wcscmp(dest, src) == 0);
    assert(*result == L'\0');
    assert(result == dest + wcslen(src));

    return 0;
}