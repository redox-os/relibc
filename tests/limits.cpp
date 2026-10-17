#include <assert.h>
#include <stdio.h>
#include <limits.h>
#include <stdint.h>
#include <sys/types.h>
#include <stdbool.h>

#define CHECK_TYPE(var, limit) \
    ([=]() { \
        if (1) return var; \
        else   return limit; \
    }())

int main() {
    int         i_max  = INT_MAX;
    int         i_min  = INT_MIN;
    long        l_max  = LONG_MAX;
    long        l_min  = LONG_MIN;
    long long   ll_max = LLONG_MAX;
    long long   ll_min = LLONG_MIN;
    ssize_t     ss_max = SSIZE_MAX;

    unsigned int       ui_max  = UINT_MAX;
    unsigned long      ul_max  = ULONG_MAX;
    unsigned long long ull_max = ULLONG_MAX;

    CHECK_TYPE(i_max, INT_MAX);
    CHECK_TYPE(i_min, INT_MIN);
    CHECK_TYPE(l_max, LONG_MAX);
    CHECK_TYPE(l_min, LONG_MIN);
    CHECK_TYPE(ll_max, LLONG_MAX);
    CHECK_TYPE(ll_min, LLONG_MIN);
    CHECK_TYPE(ss_max, SSIZE_MAX);

    CHECK_TYPE(ui_max, UINT_MAX);
    CHECK_TYPE(ul_max, ULONG_MAX);
    CHECK_TYPE(ull_max, ULLONG_MAX);

    printf("INT       : [%d, %d]\n", i_min, i_max);
    printf("LONG      : [%ld, %ld]\n", l_min, l_max);
    printf("LLONG     : [%lld, %lld]\n", ll_min, ll_max);
    printf("SSIZE_MAX : %zd\n\n", ss_max);
    printf("UINT_MAX   : %u\n", ui_max);
    printf("ULONG_MAX  : %lu\n", ul_max);
    printf("ULLONG_MAX : %llu\n\n", ull_max);

    return 0;
}
