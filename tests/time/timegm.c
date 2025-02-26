#include <time.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

int main(void) {
    struct {
        time_t timestamp;
        const char *expected_asctime;
    } test_cases[] = {
        {0, "Thu Jan  1 00:00:00 1970\n"},
        {5097600, "Sun Mar  1 00:00:00 1970\n"},
        {31535999, "Thu Dec 31 23:59:59 1970\n"},
        {68255999, "Tue Feb 29 23:59:59 1972\n"}, // 1972 is a leap year
        {94694399, "Sun Dec 31 23:59:59 1972\n"},
        {951868799, "Tue Feb 29 23:59:59 2000\n"}, // 2000 is a leap year
        {978307199, "Sun Dec 31 23:59:59 2000\n"},
        {4107542400, "Mon Mar  1 00:00:00 2100\n"}, // 2100 is not a leap year
        {4133980799, "Fri Dec 31 23:59:59 2100\n"},
        {2147483647, "Tue Jan 19 03:14:07 2038\n"}, // 32-bit overflow point
        {2147483648, "Tue Jan 19 03:14:08 2038\n"},
        {4294967295, "Sun Feb  7 06:28:15 2106\n"}, // Unsigned 32-bit overflow
        {4294967296, "Sun Feb  7 06:28:16 2106\n"}
    };
    int num_tests = sizeof(test_cases) / sizeof(test_cases[0]);

    for (int i = 0; i < num_tests; i++) {
        time_t orig = test_cases[i].timestamp;
        const char *expected_asctime = test_cases[i].expected_asctime;

        struct tm *tm_ptr = gmtime(&orig);
        assert(tm_ptr != NULL);

        struct tm local_tm;
        memcpy(&local_tm, tm_ptr, sizeof(struct tm));

        char *asc_time = asctime(&local_tm);
        assert(asc_time != NULL);
        assert(strcmp(asc_time, expected_asctime) == 0);

        time_t computed = timegm(&local_tm);
        assert(computed == orig);
    }

    return 0;
}
