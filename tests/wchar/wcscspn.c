#include <assert.h>
#include <wchar.h>

int main(void) {

    assert(wcscspn(L"", L"") == 0);
    assert(wcscspn(L"", L"h") == 0);
    assert(wcscspn(L"a", L"a") == 0);

    assert(wcscspn(L"ba", L"ab") == 0);
    assert(wcscspn(L"ba", L"a") == 1);

    assert(wcscspn(L"abcdefghijkl$\"£$%", L"zxqrst,./$w") == 12);

    return 0;
}
