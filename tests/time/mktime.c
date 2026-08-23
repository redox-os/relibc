#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "test_helpers.h"

int check(time_t input) {
    struct tm *t = localtime(&input);
    ERROR_IF(localtime, t, == NULL);

    time_t output = mktime(t);
    ERROR_IF(mktime, output, == (time_t)-1);

    printf("%ld = %ld\n", input, output);

    if (input != output) {
        printf(
            "Year %d, Day of year: %d, Month %d, Day of month: %d, Day of week: %d, %d:%d:%d\n",
            t->tm_year, t->tm_yday, t->tm_mon, t->tm_mday, t->tm_wday, t->tm_hour, t->tm_min, t->tm_sec
        );
        puts("Failed!");
        return 1;
    }
    return 0;
}

time_t try_mktime(const char* posix) {
    setenv("TZ", posix, 1);
    tzset();

    struct tm t_posix = { 0 };
    t_posix.tm_year = 2024 - 1900;
    t_posix.tm_mon = 0;
    t_posix.tm_mday = 1;
    t_posix.tm_hour = 12;
    t_posix.tm_min = 0;
    t_posix.tm_sec = 0;

    return mktime(&t_posix);
}
void check_invalid(time_t result, const char* std, long time, long _timezone) {
    if (
        result != time ||
        timezone != _timezone ||
        strcmp(tzname[0], std) != 0
    ) {
        printf(
            "ERROR: %s != %s OR %ld != %ld OR %ld != %ld\n",
            tzname[0], std,
            result, time,
            timezone, _timezone
        );
        // exit(EXIT_FAILURE);
    }
}

int main(void) {
    struct tm t = { 0 };

    t.tm_year = 71;
    t.tm_mday = 1;

    printf("%ld\n", mktime(&t));

    int day = 60 * 60 * 24;
    time_t inputs[] = { -(day * 33), -day, -500, 0, 1531454950 };
    for (int i = 0; i < 5; i += 1) {
        if (check(inputs[i])) {
            exit(EXIT_FAILURE);
        }
    }

    srand(time(NULL));

    for (int i = 0; i < 10; i += 1) {
        time_t input = (time_t) rand();

        struct tm *time = localtime(&input);
        ERROR_IF(localtime, time, == NULL);

        time_t output = mktime(time);
        ERROR_IF(mktime, output, == (time_t)-1);

        if (input != output) {
            // asctime has newline
            printf("Comparison %ld == %ld failed. Time: %s", input, output, asctime(time));
        }
    }

    // mktime must not panic on out-of-range tm_gmtoff
    {
        struct tm tg = { 0 };
        tg.tm_year = 124;
        tg.tm_mon = 0;
        tg.tm_mday = 1;
        tg.tm_hour = 12;
        tg.tm_gmtoff = 100000;
        time_t result = mktime(&tg);
        printf("gmtoff_ignored = %d\n", result != (time_t)-1);
    }

    // ===== POSIX =====
    time_t result = try_mktime("EST5");
    check_invalid(result, "EST", 1704128400, 18000);

    // No value
    result = try_mktime("");
    check_invalid(result, "UTC", 1704110400, 0);

    // Negative
    result = try_mktime("EST-5");
    check_invalid(result, "EST", 1704092400, -18000);

    // No offset
    result = try_mktime("JST");
    check_invalid(result, "JST", 1704110400, 0);

    // longer than 3 chars & fake timezone test
    result = try_mktime("SMNTH-5");
    check_invalid(result, "SMNTH", 1704092400, -18000);

    // Invalid POSIX test (shorter than 3 chars)
    result = try_mktime("SM5NTH5");
    check_invalid(result, "", 1704110400, 0);

    // Clamp test
    result = try_mktime("CST-100");
    check_invalid(result, "CST", 1704024000, -86400);

    // Explicit +
    result = try_mktime("CDT+5");
    check_invalid(result, "CDT", 1704128400, 18000);

    // Minutes included
    result = try_mktime("IST-5:30");
    check_invalid(result, "IST", 1704090600, -19800);

    // Seconds included
    result = try_mktime("FAKE5:00:30");
    check_invalid(result, "FAKE", 1704128430, 18030);

    // Explicit + with 0
    result = try_mktime("JST+0");
    check_invalid(result, "JST", 1704110400, 0);

    // Full POSIX
    result = try_mktime("EST5EDT,M3.2.0/2,M11.1.0/2");
    check_invalid(result, "EST", 1704128400, 18000);

    // POSIX without time
    result = try_mktime("EST5EDT,M3.2.0,M11.1.0");
    check_invalid(result, "EST", 1704128400, 18000);

    // dst empty test
    result = try_mktime("EST5,M3.2.0,M11.1.0");
    check_invalid(result, "EST", 1704128400, 18000);

    // end empty test
    result = try_mktime("EST5EDT,M3.2.0/2");
    check_invalid(result, "EST", 1704128400, 18000);

    // dst & end empty test
    result = try_mktime("EST5,M3.2.0");
    check_invalid(result, "EST", 1704128400, 18000);

    // start empty test
    result = try_mktime("EST5EDT,,M3.2.0/2");
    check_invalid(result, "EST", 1704128400, 18000);

    // dst & start empty test
    result = try_mktime("EST5,,M3.2.0/2");
    check_invalid(result, "EST", 1704128400, 18000);
}
