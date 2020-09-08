#include <time.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "test_helpers.h"

void print_tm(const struct tm *tm_ptr) {
    printf("  tm_sec    = %d\n", tm_ptr->tm_sec);
    printf("  tm_min    = %d\n", tm_ptr->tm_min);
    printf("  tm_hour   = %d\n", tm_ptr->tm_hour);
    printf("  tm_mday   = %d\n", tm_ptr->tm_mday);
    printf("  tm_mon    = %d\n", tm_ptr->tm_mon);
    printf("  tm_year   = %d\n", tm_ptr->tm_year);
    printf("  tm_wday   = %d\n", tm_ptr->tm_wday);
    printf("  tm_yday   = %d\n", tm_ptr->tm_yday);
    printf("  tm_isdst  = %d\n", tm_ptr->tm_isdst);
    printf("  tm_gmtoff = %ld\n", tm_ptr->tm_gmtoff);
    printf("  tm_zone   = %s\n", tm_ptr->tm_zone);
}

int main(void) {
    time_t unix_epoch_seconds = 0;

    // Exercise the different branches of the leap year logic
    // 1970: common year
    time_t y1970_mar01_seconds = 5097600;    // 1970-03-01 00:00:00
    time_t y1970_dec31_seconds = 31535999;   // 1970-12-31 23:59:59

    // 1972: leap year
    time_t y1972_feb29_seconds = 68255999;   // 1972-02-29 23:59:59
    time_t y1972_dec31_seconds = 94694399;   // 1972-12-31 23:59:59

    // 2000: leap year
    time_t y2000_feb29_seconds = 951868799;  // 2000-02-29 23:59:59
    time_t y2000_dec31_seconds = 978307199;  // 2000-12-31 23:59:59

    // 2100: common year
    time_t y2100_mar01_seconds = 4107542400; // 2100-03-01 00:00:00
    time_t y2100_dec31_seconds = 4133980799; // 2100-12-31 23:59:59

    // Year 2038-related corner cases
    time_t y2038_pre_seconds = 2147483647;
    time_t y2038_post_seconds = 2147483648;
    time_t y2106_pre_seconds = 4294967295;
    time_t y2106_post_seconds = 4294967296;

    struct tm *result_tm = NULL;

    puts("Unix epoch:");
    result_tm = gmtime(&unix_epoch_seconds);
    print_tm(result_tm);

    puts("");
    puts("1970-03-01 00:00:00:");
    result_tm = gmtime(&y1970_mar01_seconds);
    print_tm(result_tm);

    puts("");
    puts("1970-12-31 23:59:59:");
    result_tm = gmtime(&y1970_dec31_seconds);
    print_tm(result_tm);

    puts("");
    puts("1972-02-29 23:59:59:");
    result_tm = gmtime(&y1972_feb29_seconds);
    print_tm(result_tm);

    puts("");
    puts("1972-12-31 23:59:59:");
    result_tm = gmtime(&y1972_dec31_seconds);
    print_tm(result_tm);

    puts("");
    puts("2000-02-29 23:59:59:");
    result_tm = gmtime(&y2000_feb29_seconds);
    print_tm(result_tm);

    puts("");
    puts("2000-12-31 23:59:59:");
    result_tm = gmtime(&y2000_dec31_seconds);
    print_tm(result_tm);

    puts("");
    puts("2100-03-01 00:00:00:");
    result_tm = gmtime(&y2100_mar01_seconds);
    print_tm(result_tm);

    puts("");
    puts("2100-12-31 23:59:59:");
    result_tm = gmtime(&y2100_dec31_seconds);
    print_tm(result_tm);

    puts("");
    puts("Year 2038, pre-i32 overflow:");
    result_tm = gmtime(&y2038_pre_seconds);
    print_tm(result_tm);

    puts("");
    puts("Year 2038, post-i32 overflow:");
    result_tm = gmtime(&y2038_post_seconds);
    print_tm(result_tm);

    puts("");
    puts("Year 2106, pre-u32 overflow:");
    result_tm = gmtime(&y2106_pre_seconds);
    print_tm(result_tm);

    puts("");
    puts("Year 2106, post-u32 overflow:");
    result_tm = gmtime(&y2106_post_seconds);
    print_tm(result_tm);
}
