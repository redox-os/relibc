#include <locale.h>
#include <stdio.h>
#include <assert.h>
#include <stdint.h>

int main() {
    struct lconv *lconv0 = localeconv();

    locale_t locale0 = uselocale(NULL);
    assert(locale0 == LC_GLOBAL_LOCALE);

    locale_t locale1 = uselocale(LC_GLOBAL_LOCALE);
    assert(locale1 == LC_GLOBAL_LOCALE);

    struct lconv *lconv1 = localeconv();
    assert(lconv0 == lconv1);
    
    locale_t locale2 = newlocale(LC_ALL_MASK, "C", (locale_t)0);
    assert(locale2 != LC_GLOBAL_LOCALE);

    struct lconv *lconv2 = localeconv();
    assert(lconv2 == lconv1);
    
    locale_t locale3 = duplocale(locale2);
    assert(locale3 != locale2);
    assert(locale3 != LC_GLOBAL_LOCALE);

    locale_t locale4 = uselocale(locale3);
    assert(locale4 == locale1);

    struct lconv *lconv3 = localeconv();
    assert(lconv3 != lconv2);

    // should not crash
    freelocale(locale3);
    freelocale(locale2);
    freelocale(locale1);
    return 0;
}