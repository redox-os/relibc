#include <wchar.h>
#include <assert.h>
#include <string.h>
#include <stdlib.h>

int main() {
    wchar_t src[] = L"こんにちは世界Привет мир";

    wchar_t* dup = wcsdup(src);
    
    assert(wcscmp(dup, src) == 0);
    assert(dup != src);
    free(dup);

    return 0;
}
