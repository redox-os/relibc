#include <stdio.h>
#include <limits.h>
#include <stdint.h>
#include <sys/types.h>

int main() {
    char        c_max  = CHAR_MAX;
    char        c_min  = CHAR_MIN;
    signed char sc_max = SCHAR_MAX;
    signed char sc_min = SCHAR_MIN;
    short       s_max  = SHRT_MAX;
    short       s_min  = SHRT_MIN;
    int         i_max  = INT_MAX;
    int         i_min  = INT_MIN;
    long        l_max  = LONG_MAX;
    long        l_min  = LONG_MIN;
    long long   ll_max = LLONG_MAX;
    long long   ll_min = LLONG_MIN;
    ssize_t     ss_max = SSIZE_MAX;
    unsigned char      uc_max = UCHAR_MAX;
    unsigned short     us_max = USHRT_MAX;
    unsigned int       ui_max = UINT_MAX;
    unsigned long      ul_max = ULONG_MAX;
    unsigned long long ull_max = ULLONG_MAX;
    int long_bit = LONG_BIT;
    int word_bit = WORD_BIT;

    printf("CHAR      : [%d, %d]\n", c_min, c_max);
    printf("SCHAR     : [%d, %d]\n", sc_min, sc_max);
    printf("SHRT      : [%d, %d]\n", s_min, s_max);
    printf("INT       : [%d, %d]\n", i_min, i_max);
    printf("LONG      : [%ld, %ld]\n", l_min, l_max);
    printf("LLONG     : [%lld, %lld]\n", ll_min, ll_max);
    printf("SSIZE_MAX : %zd\n\n", ss_max);
    printf("UCHAR_MAX  : %u\n", uc_max);
    printf("USHRT_MAX  : %u\n", us_max);
    printf("UINT_MAX   : %u\n", ui_max);
    printf("ULONG_MAX  : %lu\n", ul_max);
    printf("ULLONG_MAX : %llu\n\n", ull_max);
    printf("LONG_BIT : %d\n", long_bit);
    printf("WORD_BIT : %d\n", word_bit);

    return 0;
}
