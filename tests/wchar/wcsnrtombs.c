#include <assert.h>
#include <locale.h>
#include <wchar.h>
#include <string.h>

int main() {
    setlocale(LC_ALL, "");

    const wchar_t *src = L"こんにちは世界Привет мир";
    char dst[20];
    mbstate_t ps;

    size_t result = wcsnrtombs(dst, &src, 4, sizeof(dst), &ps);
    assert(result == 12);
    assert(strcmp(dst, "こんにち") == 0);

    return 0;
}
