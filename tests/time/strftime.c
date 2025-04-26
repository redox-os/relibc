#include <stdlib.h>
#include <stdio.h>
#include <time.h>
#include <assert.h>
#include "test_helpers.h"

#define PRINT_BUFSZ 50
#define V_BUFSZ 3
#define BUFS 30

void print(time_t timestamp, char* fmt) {
    char out[PRINT_BUFSZ] = {0};
    size_t n = strftime(out, PRINT_BUFSZ, fmt, localtime(&timestamp));
    printf("%zu: %s\n", n, out);

}

void strftime_mix(void) {
    setenv("TZ", "Australia/Melbourne", 1);
    tzset();

    // Initialize a struct tm with a known fixed date and time.
    // December 31, 2020 23:59:59, which is a Thursday.
    struct tm t = {0};
    t.tm_year = 2020 - 1900;   // Years since 1900.
    t.tm_mon  = 11;            // December (0-based, so 11 means December).
    t.tm_mday = 31;
    t.tm_hour = 23;
    t.tm_min  = 59;
    t.tm_sec  = 59;
    t.tm_wday = 4;             // Thursday (0 = Sunday, so Thursday = 4).
    t.tm_yday = 365;           
    t.tm_gmtoff = 39600;       // GMT offset for AEDT (UTC+11).
    t.tm_isdst = 11;           // Daylight saving time is in effect.
    t.tm_zone = "AEDT";        // Time zone name.
    char buf[BUFS];

    strftime(buf, BUFS, "%a", &t);
    assert(strcmp(buf, "Thu") == 0);

    strftime(buf, BUFS, "%A", &t);
    assert(strcmp(buf, "Thursday") == 0);

    strftime(buf, BUFS, "%b", &t);
    assert(strcmp(buf, "Dec") == 0);

    strftime(buf, BUFS, "%B", &t);
    assert(strcmp(buf, "December") == 0);

    strftime(buf, BUFS, "%d", &t);
    assert(strcmp(buf, "31") == 0);

    strftime(buf, BUFS, "%H", &t);
    assert(strcmp(buf, "23") == 0);

    strftime(buf, BUFS, "%I", &t);
    assert(strcmp(buf, "11") == 0);

    // Day of the year as a zero-padded decimal number (001-366).
    strftime(buf, BUFS, "%j", &t);
    // Even though t.tm_yday is 365 (zero-based), %j is one-based: "366"
    assert(strcmp(buf, "366") == 0);

    strftime(buf, BUFS, "%m", &t);
    assert(strcmp(buf, "12") == 0);

    strftime(buf, BUFS, "%M", &t);
    assert(strcmp(buf, "59") == 0);

    strftime(buf, BUFS, "%p", &t);
    assert(strcmp(buf, "PM") == 0);

    strftime(buf, BUFS, "%S", &t);
    assert(strcmp(buf, "59") == 0);

    strftime(buf, BUFS, "%U", &t);
    assert(strcmp(buf, "52") == 0);

    strftime(buf, BUFS, "%w", &t);
    assert(strcmp(buf, "4") == 0);

    strftime(buf, BUFS, "%W", &t);
    assert(strcmp(buf, "52") == 0);

    strftime(buf, BUFS, "%y", &t);
    assert(strcmp(buf, "20") == 0);

    strftime(buf, BUFS, "%Y", &t);
    assert(strcmp(buf, "2020") == 0);

    strftime(buf, BUFS, "%Z", &t);
    assert(strcmp(buf, "AEDT") == 0);

    strftime(buf, BUFS, "%z", &t);
    assert(strcmp(buf, "+1100") == 0);

    strftime(buf, BUFS, "%F", &t);
    assert(strcmp(buf, "2020-12-31") == 0);

    strftime(buf, BUFS, "%R", &t);
    assert(strcmp(buf, "23:59") == 0);

    strftime(buf, BUFS, "%r", &t);
    assert(strcmp(buf, "11:59:59 PM") == 0);

    strftime(buf, BUFS, "%T", &t);
    assert(strcmp(buf, "23:59:59") == 0);
}

void test_v(void) {
    char buf[V_BUFSZ] = {0};
    struct tm time = {0};

    // Example dates copied from Wikipedia
    // Saturday 2005-01-01
    time.tm_yday = 0;
    time.tm_wday = 6;
    time.tm_year = 2005;
    strftime(buf, V_BUFSZ, "%V", &time);
    puts(buf);

    // Saturday 2005-12-31
    time.tm_yday = 365;
    time.tm_wday = 6;
    strftime(buf, V_BUFSZ, "%V", &time);
    puts(buf);

    // Sunday 2006-01-01
    time.tm_yday = 0;
    time.tm_wday = 0;
    time.tm_year = 2006;
    strftime(buf, V_BUFSZ, "%V", &time);
    puts(buf);

    // Sunday 2008-12-28
    time.tm_yday = 362;
    time.tm_wday = 0;
    time.tm_year = 2008;
    strftime(buf, V_BUFSZ, "%V", &time);
    puts(buf);

    // Friday 2010-01-01
    time.tm_yday = 0;
    time.tm_wday = 5;
    time.tm_year = 2010;
    strftime(buf, V_BUFSZ, "%V", &time);
    puts(buf);
}

int main(void) {
    print(1531808742, "%a %A %b %B");
    print(1531808742, "The %Cst century");
    print(1531808742, "%I:%M:%S %p");
    print(1531839600, "%r");
    print(1531839600, "%R");
    print(1531839600, "%H %s %u");
    print(1531839600, "%j %U");
    print(1531839600, "%+");
    print(1533669431, "%+%+%+%+%+"); // will overflow 50 characters

    // ISO-8601 tests
    test_v();

    strftime_mix();

    return EXIT_SUCCESS;
}
