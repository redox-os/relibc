#define _XOPEN_SOURCE 700

#include <assert.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "test_helpers.h"

__attribute__((nonnull))
static void strptime_test(const char* restrict time,
                          const char* restrict format,
                          struct tm expected,
                          const char* restrict exp_str) {
    struct tm actual = {0};
    const char* result = strptime(time, format, &actual);

    // This struct is packed and zeroed beforehand so it shouldn't
    // have any holes to throw off memcmp.
    //
    // If relibc implements the GNU extensions that store timezone,
    // then this will need to be modified to memcmp up to
    // sizeof(struct tm) - sizeof(char*) followed by a strcmp.
    //
    // glibc and musl differ a bit which is why some calls to
    // this function comment out tm_yday and tm_wday.
    // glibc seems to set these fields even if they're not specified,
    // which is a cool extra but implementation specific behavior.
    if(memcmp(&expected, &actual, sizeof(struct tm))) {
        puts("struct tm expected versus actual\n");
        printf("%-8s %d %4d\n", "tm_year", expected.tm_year, actual.tm_year);
        printf("%-8s %d %4d\n", "tm_mon", expected.tm_mon, actual.tm_mon);
        printf("%-8s %d %4d\n", "tm_mday", expected.tm_mday, actual.tm_mday);
        printf("%-8s %d %4d\n", "tm_hour", expected.tm_hour, actual.tm_hour);
        printf("%-8s %d %4d\n", "tm_min", expected.tm_min, actual.tm_min);
        printf("%-8s %d %4d\n", "tm_sec", expected.tm_sec, actual.tm_sec);
        printf("%-8s %d %4d\n", "tm_wday", expected.tm_wday, actual.tm_wday);
        printf("%-8s %d %4d\n", "tm_yday", expected.tm_yday, actual.tm_yday);
        printf("%-8s %d %4d\n", "tm_isdst", expected.tm_isdst, actual.tm_isdst);
        
        exit(EXIT_FAILURE);
    }
    
    // Safety:
    // `time` our static string which definitely ends with NUL
    // `result` is from strptime and ends with a NUL because `time` does
    size_t diff = strlen(time) - strlen(result);
    assert(!strncmp(exp_str, result, diff));
}

int main(void) {
    const char emi[] = "02:24:14";
    struct tm emi_expect = {
        .tm_hour = 2,
        .tm_min = 24,
        .tm_sec = 14,
    };
    strptime_test(emi, "%T", emi_expect, "");

    const char daydream[] = "1981-11-18 Daydream Nation";
    struct tm daydream_expect = {
        .tm_year = 81,
        .tm_mon = 10,
        .tm_mday = 18,
        /* .tm_wday = 3, */
        /* .tm_yday = 321, */
    };
    const char* daydream_rem = &daydream[10];
    strptime_test(daydream,
                  "%Y-%m-%d",
                  daydream_expect,
                  daydream_rem
    );

    // strptime(3): "(This is the American style date, very confusing to
    // non-Americans [...])"
    const char america[] = "07/04/76 AMERICA! (Can't use 1776 here :( ))";
    struct tm america_expect = {
        .tm_year = 76,
        .tm_mon = 6,
        .tm_mday = 4,
        /* .tm_yday = 185, */
    };
    const char* america_rem = &america[9];
    strptime_test(america,
                  "%D%n",
                  america_expect,
                  america_rem
    );

    const char percent[] = "%";
    struct tm percent_expect = {0};
    strptime_test(percent,
                  "%%",
                  percent_expect,
                  ""
    );

    // TODO: Locale offset
    const char redox[] = "Mon Oct 31 11:19:57 2016";
    struct tm redox_expect = {
        .tm_year = 116,
        .tm_mon = 9,
        .tm_mday = 31,
        .tm_hour = 11,
        .tm_min = 19,
        .tm_sec = 57,
        /* .tm_yday = 304, */
        .tm_wday = 1,
    };
    strptime_test(redox,
                  "%a%t%b%t%d%t%T%t%Y",
                  redox_expect,
                  "");

    // Roundtrip
    const char roundtrip[] = "2012-01-19 13:37:00";
    const char roundtrip_fmt[] = "%Y-%m-%d %H:%M:%S";
    struct tm roundtrip_tm = {0};
    strptime(roundtrip, roundtrip_fmt, &roundtrip_tm);

    char rt_actual[32];
    size_t rt_res = strftime(rt_actual, 32, roundtrip_fmt, &roundtrip_tm);
    size_t rt_len = strnlen(roundtrip, 32);
    assert(rt_res == rt_len);
    assert(!strncmp(roundtrip, rt_actual, rt_len));

    return EXIT_SUCCESS;
}
