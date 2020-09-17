#include <time.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "test_helpers.h"

int main(void) {
    time_t unix_epoch = 0;
    struct tm *unix_epoch_tm_ptr = gmtime(&unix_epoch);

    char *time_string = NULL;

    /* Min/max non-UB-causing values according to ISO C11 and newer. */
    struct tm iso_c_min_tm = {.tm_sec = 0, .tm_min = 0, .tm_hour = 0, .tm_mday = 1, .tm_mon = 0, .tm_year = 1000-1900, .tm_wday = 0, .tm_yday = 0, .tm_isdst = 0, .tm_gmtoff = 0, .tm_zone = NULL};
    struct tm iso_c_max_tm = {.tm_sec = 60, .tm_min = 59, .tm_hour = 23, .tm_mday = 31, .tm_mon = 11, .tm_year = 9999-1900, .tm_wday = 6, .tm_yday = 0, .tm_isdst = 0, .tm_gmtoff = 0, .tm_zone = NULL};

    /* Min/max non-UB-causing values according to POSIX (issue 7). These
     * will cause UB according to the ISO standard! */
    struct tm posix_min_tm = {.tm_sec = 0, .tm_min = 0, .tm_hour = 0, .tm_mday = -99, .tm_mon = 0, .tm_year = -999-1900, .tm_wday = 0, .tm_yday = 0, .tm_isdst = 0, .tm_gmtoff = 0, .tm_zone = NULL};
    struct tm posix_max_tm = {.tm_sec = 99, .tm_min = 99, .tm_hour = 99, .tm_mday = 999, .tm_mon = 11, .tm_year = 9999-1900, .tm_wday = 6, .tm_yday = 0, .tm_isdst = 0, .tm_gmtoff = 0, .tm_zone = NULL};

    time_string = asctime(unix_epoch_tm_ptr);
    printf("%s", time_string);

    time_string = asctime(&iso_c_min_tm);
    printf("%s", time_string);

    time_string = asctime(&iso_c_max_tm);
    printf("%s", time_string);

    time_string = asctime(&posix_min_tm);
    printf("%s", time_string);

    time_string = asctime(&posix_max_tm);
    printf("%s", time_string);

    return 0;
}
