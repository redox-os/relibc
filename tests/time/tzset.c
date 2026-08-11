#include <string.h>
#include <time.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

extern int daylight;
extern long timezone;
extern char *tzname[2];

void try_tzset(const char* zone) {
    setenv("TZ", zone, 1);
    tzset();
    printf("%s: tzname[0] %s, tzname[1] %s, daylight %d, timezone %ld\n",
        zone, tzname[0], tzname[1], daylight, timezone);
}

void check_invalid(const char* src, const char* dst, int _daylight, long _timezone) {
    if (
        strcmp(src, tzname[0]) != 0 ||
        strcmp(dst, tzname[1]) != 0 ||
        daylight != _daylight ||
        timezone != _timezone
    ) {
        printf(
            "FAILURE: Expected  - src:%s, dst:%s, daylight:%d, timezone:%ld\n",
            src, dst, _daylight, _timezone
        );
        printf(
            "\t   VALUES  - src:%s, dst:%s, daylight:%d, timezone:%ld\n",
            tzname[0], tzname[1], daylight, timezone
        );
        _exit(EXIT_FAILURE);
    }
}

int main(void) {
    // Default system value. Unmodifed Redox is usually UTC
    tzset();
    printf("Default: tzname[0] %s, tzname[1] %s, daylight %d, timezone %ld\n",
        tzname[0], tzname[1], daylight, timezone);

    try_tzset("UTC");
    check_invalid("UTC", "UTC", 0, 0);

    try_tzset("EST");
    check_invalid("EST", "EST", 0, 18000);

    // Relibc output & libc outputs are different
    // try_tzset("Japan");
    // check_invalid("JST", "JDT", 1, -32400); // Relibc: "JST", "JST", 0, -32400

    unsetenv("TZ");
    tzset();
    printf("Default after unset: tzname[0] %s, tzname[1] %s, daylight %d, timezone %ld\n",
        tzname[0], tzname[1], daylight, timezone);

    // ===== POSIX =====

    // Simple POSIX value
    try_tzset("EST5");
    check_invalid("EST", "EST", 0, 18000);

    // No value [Interesting case]
    try_tzset("");
    check_invalid("UTC", "UTC", 0, 0);

    // no offset test
    try_tzset("JST");
    check_invalid("JST", "", 0, 0);

    // longer than 3 chars & fake timezone test
    try_tzset("SMNTH-5");
    check_invalid("SMNTH", "SMNTH", 0, -18000);

    // Invalid POSIX test (shorter than 3 chars)
    try_tzset("SM5NTH5");
    check_invalid("", "", 0, 0);

    // Clamp test
    try_tzset("CST-100");
    check_invalid("CST", "CST", 0, -86400);

    // Explicit +
    try_tzset("CDT+5");
    check_invalid("CDT", "CDT", 0, 18000);

    // Minutes included
    try_tzset("IST-5:30");
    check_invalid("IST", "IST", 0, -19800);

    // Seconds included
    try_tzset("FAKE5:00:30");
    check_invalid("FAKE", "FAKE", 0, 18030);

    // Explicit + with 0
    try_tzset("JST+0");
    check_invalid("JST", "JST", 0, 0);

    // full POSIX
    try_tzset("EST5EDT,M3.2.0/2,M11.1.0/2");
    check_invalid("EST", "EDT", 1, 18000);

    // // POSIX without time
    try_tzset("EST5EDT,M3.2.0,M11.1.0");
    check_invalid("EST", "EDT", 1, 18000);

    // dst empty test
    try_tzset("EST5,M3.2.0,M11.1.0");
    check_invalid("EST", "", 1, 18000);

    // end empty test
    try_tzset("EST5EDT,M3.2.0/2");
    check_invalid("EST", "EDT", 1, 18000);

    // dst & end empty test
    try_tzset("EST5,M3.2.0");
    check_invalid("EST", "", 1, 18000);

    // start empty test
    try_tzset("EST5EDT,,M3.2.0/2");
    check_invalid("EST", "EDT", 1, 18000);

    // dst & start empty test
    try_tzset("EST5,,M3.2.0/2");
    check_invalid("EST", "", 1, 18000);

    return 0;
}
