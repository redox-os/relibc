#include <stdio.h>
#include <wchar.h>

int main() {
    wchar_t wcs[] = L"Hello    from the     otter\tslide.\nMust have    gone  down a\t\t\t \n thousand tiiiiiimeeeees...\n\n\n";
    wchar_t *token;
    wchar_t *state;
    for (token = wcstok(wcs, L" \t\n", &state);
         token != NULL;
         token = wcstok(NULL, L" \t\n", &state)) {
        fputws(token, stdout);
        fputwc(L'\n', stdout);
    }
}
