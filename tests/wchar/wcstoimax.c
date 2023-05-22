#include <inttypes.h>
#include <stdio.h>
#include <wchar.h>

int main(void) {
    wchar_t* endptr;
    wprintf(L"%ld\n", wcstoimax(L" -123junk", &endptr, 10)); /* base 10                    */
    wprintf(L"%ld\n", wcstoimax(L"11111111", &endptr, 2));   /* base 2                     */
    wprintf(L"%ld\n", wcstoimax(L"XyZ", &endptr, 36));       /* base 36                    */
    wprintf(L"%ld\n", wcstoimax(L"010", &endptr, 0));        /* octal auto-detection       */
    wprintf(L"%ld\n", wcstoimax(L"10", &endptr, 0));         /* decimal auto-detection     */
    wprintf(L"%ld\n", wcstoimax(L"0x10", &endptr, 0));       /* hexadecimal auto-detection */
}