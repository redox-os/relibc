#include <locale.h>
#include <stdio.h>
#include <assert.h>
#include <stdint.h>

int main() {    
    locale_t locale0 = newlocale(LC_ALL_MASK, "C", (locale_t)0);
    assert(locale0 != (locale_t)0);

    locale_t locale1 = newlocale(LC_ALL_MASK, "non-existent-locale", (locale_t)0);
    assert(locale1 == (locale_t)0);

    // TODO: test with existing custom locale

    return 0;
}
