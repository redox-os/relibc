#include <stdlib.h>

#include "test_helpers.h"

volatile float f;
volatile long double ld;
volatile unsigned long long ll;
lldiv_t mydivt;

int main(void) {
    char* tmp;
    f = strtof("gnu", &tmp);
    ld = strtold("gnu", &tmp);
    ll = strtoll("gnu", &tmp, 10);
    ll = strtoull("gnu", &tmp, 10);
    ll = llabs(10);
    mydivt = lldiv(10,1);
    ll = mydivt.quot;
    ll = mydivt.rem;
    ll = atoll("10");
    _Exit(0);
}

