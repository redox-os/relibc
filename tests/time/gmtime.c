#include <time.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    time_t a = 0;
    struct tm expected = { .tm_sec = 0, .tm_min = 0, .tm_hour = 0, .tm_mday = 1, .tm_year = 70,
                    .tm_wday = 4, .tm_yday = 0, .tm_isdst = 0, .tm_gmtoff = 0, .tm_zone = "UTC" };

    struct tm *info = gmtime(&a);
    if (info->tm_sec != expected.tm_sec || info->tm_min != expected.tm_min ||
        info->tm_hour != expected.tm_hour || info->tm_mday != expected.tm_mday ||
        info->tm_year != expected.tm_year || info->tm_wday != expected.tm_wday ||
        info->tm_yday != expected.tm_yday || info->tm_isdst != expected.tm_isdst ||
        info->tm_gmtoff != expected.tm_gmtoff || strcmp(info->tm_zone, expected.tm_zone) != 0) {
            return EXIT_FAILURE;
    }

    if (info->tm_sec != expected.tm_sec || info->tm_min != expected.tm_min ||
        info->tm_hour != expected.tm_hour || info->tm_mday != expected.tm_mday ||
        info->tm_year != expected.tm_year || info->tm_wday != expected.tm_wday ||
        info->tm_yday != expected.tm_yday || info->tm_isdst != expected.tm_isdst ||
        info->tm_gmtoff != expected.tm_gmtoff || strcmp(info->tm_zone, expected.tm_zone) != 0) {
            return EXIT_FAILURE;
    }
}
