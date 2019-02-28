#include <assert.h>
#include <wchar.h>

int main() {
    wchar_t *s;

    assert(wcsrchr(L"", L'a') == NULL);
    
    s = L"a";
    assert(wcsrchr(s, L'a') == s);

    s = L"aa";
    assert(wcsrchr(s, L'a') == s + 1);

    s = L"aab";
    assert(wcsrchr(s, L'a') == s + 1);
    
    s = L"abcdef!\"£$%^e&*";
    assert(wcsrchr(s, L'g') == NULL);
    assert(wcsrchr(s, L'\"') == s + 7);
    assert(wcsrchr(s, L'e') == s + 12);

    return 0;
}
