#include <assert.h>
#include <wchar.h>

int main(void) {
    wchar_t *haystack = L"Hello, World!";
    wchar_t *haystack_empty = L"";

    wchar_t *needle_expected = L"World";
    wchar_t *needle_expected_multiple = L"l";
    wchar_t *needle_not_expected = L"Rust";
    wchar_t *needle_too_long = L"Hello, World!!!";
    wchar_t *needle_empty = L"";

    assert(wcsstr(haystack, needle_expected) == haystack + 7);
    assert(wcsstr(haystack, needle_expected_multiple) == haystack + 2);
    assert(wcsstr(haystack, needle_not_expected) == NULL);
    assert(wcsstr(haystack, needle_too_long) == NULL);
    assert(wcsstr(haystack, needle_empty) == haystack);

    assert(wcsstr(haystack_empty, needle_expected) == NULL);
    assert(wcsstr(haystack_empty, needle_expected_multiple) == NULL);
    assert(wcsstr(haystack_empty, needle_not_expected) == NULL);
    assert(wcsstr(haystack_empty, needle_too_long) == NULL);
    assert(wcsstr(haystack_empty, needle_empty) == haystack_empty);

    return 0;
}
