#include <locale.h>
#include <stdio.h>
#include <assert.h>
#include <stdint.h>

int main() {    
    locale_t locale0 = newlocale(LC_ALL_MASK, "C", (locale_t)0);
    assert(locale0 != (locale_t)0);

    locale_t locale1 = newlocale(LC_ALL_MASK, "non-existent-locale", (locale_t)0);
    assert(locale1 == (locale_t)0);

    // TODO: locale files inside redoxer (linux and redox)?
    // locale_t locale2 = newlocale(LC_ALL_MASK, "en_US", (locale_t)0);
    // assert(locale2 != (locale_t)0);

    return 0;
}
