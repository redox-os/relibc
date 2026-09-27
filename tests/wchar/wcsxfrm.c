#include <assert.h>
#include <locale.h>
#include <stddef.h>
#include <wchar.h>

int main(void) {

    {
        wchar_t *src = L"hello";
        wchar_t dst[10];
        assert(wcsxfrm(dst, src, 10) == 5);
        assert(wcscmp(dst, src) == 0);
    }

    {
        wchar_t *src = L"hello";
        wchar_t *dst = NULL;
        assert(wcsxfrm(dst, src, 0) == 5);
    }

    {
        wchar_t *src = L"";
        wchar_t dst[5] = L"ABCD";
        assert(wcsxfrm(dst, src, 5) == 0);
        assert(dst[0] == 0);
    }

    {
        wchar_t *src = L"hello";
        wchar_t dst[3];
        assert(wcsxfrm(dst, src, 3) == 5);
    }

    {
        wchar_t *src = L"hello";
        wchar_t dst[6];
        assert(wcsxfrm(dst, src, 6) == 5);
        assert(wcscmp(dst, src) == 0);
    }

    {
        wchar_t *src = L"hello";
        wchar_t dst[5] = L"ABCD";
        assert(wcsxfrm(dst, src, 5) == 5);
        assert(wcscmp(dst, L"ABCD") == 0);
    }

    {
        wchar_t *src = L"héllo世界";
        wchar_t dst[10];
        assert(wcsxfrm(dst, src, 8) == 7);
        assert(wcscmp(dst, src) == 0);
    }

    {
        wchar_t *src = L"hello";
        wchar_t dst[10];
        assert(wcsxfrm_l(dst, src, 10, LC_GLOBAL_LOCALE) == 5);
        assert(wcscmp(dst, src) == 0);
    }

    {
        wchar_t *src = L"hello";
        wchar_t dst[10] = L"ABCDEFGHI";

        size_t len = wcsxfrm(dst, src, 0);
        assert(len == 5);
        assert(wcscmp(dst, L"ABCDEFGHI") == 0);

        assert(wcsxfrm(dst, src, len + 1) == len);
        assert(wcscmp(dst, src) == 0);
    }

    return 0;
}
