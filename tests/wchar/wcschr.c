#include <assert.h>
#include <wchar.h>

int main(void) {
    wchar_t *haystack = L"Hello World!";

    assert(wcschr(haystack, L'H') == haystack);
    assert(wcschr(haystack, L'W') == &haystack[6]);
    assert(wcschr(haystack, L'\0') == &haystack[12]); // the terminating nul is considered part of the string
    assert(wcschr(haystack, L'X') == NULL);

    return 0;
}
