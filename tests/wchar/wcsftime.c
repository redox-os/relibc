#include <stdlib.h>
#include <stdio.h>
#include <time.h>
#include <assert.h>
#include <wchar.h>
#include "test_helpers.h"

#define PRINT_BUFSZ 50
#define V_BUFSZ 3
#define BUFS 30

void print(time_t timestamp, wchar_t* fmt) {
    wchar_t out[PRINT_BUFSZ] = {0};
    size_t n = wcsftime(out, PRINT_BUFSZ, fmt, localtime(&timestamp));
    wprintf(L"%zu: %ls\n", n, out);

}

void wcsftime_mix(void) {
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
    wchar_t buf[BUFS];

    wcsftime(buf, BUFS, L"%a", &t);
    assert(wcscmp(buf, L"Thu") == 0);

    wcsftime(buf, BUFS, L"%A", &t);
    assert(wcscmp(buf, L"Thursday") == 0);

    wcsftime(buf, BUFS, L"%b", &t);
    assert(wcscmp(buf, L"Dec") == 0);

    wcsftime(buf, BUFS, L"%B", &t);
    assert(wcscmp(buf, L"December") == 0);

    wcsftime(buf, BUFS, L"%d", &t);
    assert(wcscmp(buf, L"31") == 0);

    wcsftime(buf, BUFS, L"%H", &t);
    assert(wcscmp(buf, L"23") == 0);

    wcsftime(buf, BUFS, L"%I", &t);
    assert(wcscmp(buf, L"11") == 0);

    // Day of the year as a zero-padded decimal number (001-366).
    wcsftime(buf, BUFS, L"%j", &t);
    // Even though t.tm_yday is 365 (zero-based), %j is one-based: "366"
    assert(wcscmp(buf, L"366") == 0);

    wcsftime(buf, BUFS, L"%m", &t);
    assert(wcscmp(buf, L"12") == 0);

    wcsftime(buf, BUFS, L"%M", &t);
    assert(wcscmp(buf, L"59") == 0);

    wcsftime(buf, BUFS, L"%p", &t);
    assert(wcscmp(buf, L"PM") == 0);

    wcsftime(buf, BUFS, L"%S", &t);
    assert(wcscmp(buf, L"59") == 0);

    wcsftime(buf, BUFS, L"%U", &t);
    assert(wcscmp(buf, L"52") == 0);

    wcsftime(buf, BUFS, L"%w", &t);
    assert(wcscmp(buf, L"4") == 0);

    wcsftime(buf, BUFS, L"%W", &t);
    assert(wcscmp(buf, L"52") == 0);

    wcsftime(buf, BUFS, L"%y", &t);
    assert(wcscmp(buf, L"20") == 0);

    wcsftime(buf, BUFS, L"%Y", &t);
    assert(wcscmp(buf, L"2020") == 0);

    wcsftime(buf, BUFS, L"%Z", &t);
    assert(wcscmp(buf, L"AEDT") == 0);

    wcsftime(buf, BUFS,L"%z", &t);
    assert(wcscmp(buf, L"+1100") == 0);

    wcsftime(buf, BUFS, L"%F", &t);
    assert(wcscmp(buf, L"2020-12-31") == 0);

    wcsftime(buf, BUFS, L"%R", &t);
    assert(wcscmp(buf, L"23:59") == 0);

    wcsftime(buf, BUFS, L"%r", &t);
    assert(wcscmp(buf, L"11:59:59 PM") == 0);

    wcsftime(buf, BUFS, L"%T", &t);
    assert(wcscmp(buf, L"23:59:59") == 0);

    wcsftime(buf, BUFS, L"%x", &t);
    assert(wcscmp(buf, L"12/31/20") == 0);

    wcsftime(buf, BUFS, L"%X", &t);
    assert(wcscmp(buf, L"23:59:59") == 0);

    {
        wchar_t small[4];
        assert(wcsftime(small, 4, L"%Y", &t) == 0);
    }
    {
        wchar_t exact[5];
        assert(wcsftime(exact, 5, L"%Y", &t) == 4);
        assert(wcscmp(exact, L"2020") == 0);
    }
    assert(wcsftime(buf, 0, L"%Y", &t) == 0);
}

void test_v(void) {
    wchar_t buf[V_BUFSZ] = {0};
    struct tm time = {0};

    // Example dates copied from Wikipedia
    // Saturday 2005-01-01
    time.tm_yday = 0;
    time.tm_wday = 6;
    time.tm_year = 2005;
    wcsftime(buf, V_BUFSZ, L"%V", &time);
    wprintf(L"%ls\n", buf);

    // Saturday 2005-12-31
    time.tm_yday = 365;
    time.tm_wday = 6;
    wcsftime(buf, V_BUFSZ, L"%V", &time);
    wprintf(L"%ls\n", buf);

    // Sunday 2006-01-01
    time.tm_yday = 0;
    time.tm_wday = 0;
    time.tm_year = 2006;
    wcsftime(buf, V_BUFSZ, L"%V", &time);
    wprintf(L"%ls\n", buf);

    // Sunday 2008-12-28
    time.tm_yday = 362;
    time.tm_wday = 0;
    time.tm_year = 2008;
    wcsftime(buf, V_BUFSZ, L"%V", &time);
    wprintf(L"%ls\n", buf);

    // Friday 2010-01-01
    time.tm_yday = 0;
    time.tm_wday = 5;
    time.tm_year = 2010;
    wcsftime(buf, V_BUFSZ, L"%V", &time);
    wprintf(L"%ls\n", buf);
}

int main(void) {
    setenv("TZ", "UTC", 1);
    tzset();
    print(1531808742, L"%a %A %b %B");
    print(1531808742, L"The %Cst century");
    print(1531808742, L"%I:%M:%S %p");
    print(1531839600, L"%r");
    print(1531839600, L"%R");
    print(1531839600, L"%H %s %u");
    print(1531839600, L"%j %U");
    print(1531839600, L"%+");
    print(1533669431, L"%+%+%+%+%+"); // will overflow 50 characters

    // ISO-8601 tests
    test_v();

    wcsftime_mix();

    return EXIT_SUCCESS;
}
