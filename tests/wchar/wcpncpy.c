#include <wchar.h>
#include <assert.h>
#include <string.h>

int main() {
    wchar_t src[] = L"Привет мир";

    size_t n_input = 10;
    size_t n_short = 6;
    size_t n_long = 15;

    // Initialize with sentinel values to detect exactly how much is overwritten
    wchar_t dest_short[] = L"\x12\x34\x56\x78\x90\x12\x34\x56\x78\x90\x12\x34\x56\x78";
    wchar_t dest_long[] = L"\x12\x34\x56\x78\x90\x12\x34\x56\x78\x90\x12\x34\x56\x78";

    wchar_t expected_short[] = L"Привет\x34\x56\x78\x90\x12\x34\x56\x78";

    // The "short" test should copy exactly n_short characters without terminating null
    wchar_t* result_short = wcpncpy(dest_short, src, n_short);
    assert(wcsncmp(dest_short, src, n_short) == 0);
    assert(result_short == dest_short + n_short);
    for (size_t i = n_short; i < n_long; i++) {
        // Check that the sentinel characters have not been overwritten
        assert(dest_short[i] == expected_short[i]);
    }

    // The "long" test should write the input string, with nulls appended up to n_long
    wchar_t* result_long = wcpncpy(dest_long, src, n_long);
    assert(wcsncmp(dest_long, src, n_long) == 0);
    assert(result_long == dest_long + n_input);
    for (size_t i = n_input; i < n_long; i++) {
        // Check that nulls are written up to n_long
        assert(dest_long[i] == L'\0');
    }

    return 0;
}
