#include <assert.h>
#include <time.h>
#include <string.h>
#include <stdlib.h>

void test_localtime_r_epoch() {
    time_t t = 0;  // Unix epoch (1970-01-01 00:00:00)
    struct tm result;

    assert(localtime_r(&t, &result) != NULL); 
    assert(result.tm_year == 70);   // Year 1970 - 1900
    assert(result.tm_mon == 0);     // January
    assert(result.tm_mday == 1);    // 1st of January
    assert(result.tm_hour == 0);    // Midnight
    assert(result.tm_min == 0);     
    assert(result.tm_sec == 0);     

    assert(result.tm_isdst == -1 || result.tm_isdst == 0);
}

void test_localtime_r_non_epoch() {
    time_t t = 1609459200;  // January 1, 2021 00:00:00 UTC (New Year 2021)
    struct tm result;

    assert(localtime_r(&t, &result) != NULL);
    assert(result.tm_year == 121);  // Year 2021 - 1900
    assert(result.tm_mon == 0);    // January
    assert(result.tm_mday == 1);   // 1st of January
    assert(result.tm_hour == 0);   // Midnight
    assert(result.tm_min == 0);
    assert(result.tm_sec == 0);
}

void test_localtime_r_dst() {
    time_t t = 1615708800;  // March 14 2021 08:00:00 UTC
    struct tm result;

    assert(localtime_r(&t, &result) != NULL);  // Ensure localtime_r does not fail
    assert(result.tm_year == 121);  // Year 2021 - 1900
    assert(result.tm_mon == 2);    // March
    assert(result.tm_mday == 14);  // 14th
    assert(result.tm_hour == 8);   // 08:00:00 local time
    assert(result.tm_min == 0);
    assert(result.tm_sec == 0);
}

void test_localtime_r_large_time() {
    time_t t = 32503680000;  // A large value: 1 January 3000 (UTC)
    struct tm result;

    assert(localtime_r(&t, &result) != NULL);  // Ensure localtime_r does not fail
    assert(result.tm_year == 1100);  // Year 3000 - 1900 = 1100
    assert(result.tm_mon == 0);     // January
    assert(result.tm_mday == 1);    // 1st of January
    assert(result.tm_hour == 0);    // Midnight
    assert(result.tm_min == 0);
    assert(result.tm_sec == 0);
}

void test_dst_transition() {
    time_t t = 1615809600;  // March 15 2021, 08:00:00 UTC
    struct tm result;

    setenv("TZ", "America/New_York", 1); 
    tzset();

    localtime_r(&t, &result);

    assert(result.tm_year == 121);  // Year 2021 - 1900 = 121
    assert(result.tm_mon == 2);     // March (0-based index)
    assert(result.tm_mday == 15);    // 15th
    assert(result.tm_hour == 8);    // 00:00:00 local time
    assert(result.tm_min == 0);
    assert(result.tm_sec == 0);

    assert(result.tm_isdst == 1);  // DST should be active
    
    assert(strcmp(tzname[0], "EST") == 0 || strcmp(tzname[0], "EDT") == 0);  // Standard or DST name
    assert(strcmp(tzname[1], "EDT") == 0);  // Should be the DST version of the timezone
}

void test_standard_time() {
    time_t t = 1609459200;  // January 1, 2021, 00:00:00 UTC
    struct tm result;

    setenv("TZ", "Asia/Tokyo", 1);
    tzset();

    localtime_r(&t, &result);

    assert(result.tm_year == 121);  // Year 2021 - 1900 = 121
    assert(result.tm_mon == 0);     // January (0-based index)
    assert(result.tm_mday == 1);    // 1st
    assert(result.tm_hour == 9);    // 09:00:00 local time
    assert(result.tm_min == 0);
    assert(result.tm_sec == 0);

    assert(result.tm_isdst == 0);  // DST should NOT be active
    
    assert(strcmp(tzname[0], "JST") == 0);  // Standard time name
    assert(strcmp(tzname[1], "EST") != 0);  // Should NOT be in DST
}

int main() {
    test_localtime_r_epoch();
    test_localtime_r_non_epoch();
    test_localtime_r_dst();
    test_localtime_r_large_time();
    test_dst_transition();
    test_standard_time();
    return 0;
}
