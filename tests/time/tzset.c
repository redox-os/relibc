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

int main(void) {
    // Default value is UTC for me
    tzset();
    printf("Default: tzname[0] %s, tzname[1] %s, daylight %d, timezone %ld\n",
        tzname[0], tzname[1], daylight, timezone);

    try_tzset("UTC");
    if (strcmp("UTC", tzname[0]) != 0) {
        printf("FAILURE: zone (UTC) != tzname[0] (%s)\n", tzname[0]);
        _exit(EXIT_FAILURE);
    }

    try_tzset("EST");
    if (strcmp("EST", tzname[0]) != 0) {
        printf("FAILURE: zone (EST) != tzname[0] (%s)\n", tzname[0]);
        _exit(EXIT_FAILURE);
    }


    try_tzset("Japan");
    if (strcmp("JST", tzname[0]) != 0) {
        printf("FAILURE: zone (JST) != tzname[0] (%s)\n", tzname[0]);
        _exit(EXIT_FAILURE);
    }

    // ===== POSIX =====
    
    try_tzset("EST5");
    if (strcmp("EST", tzname[0]) != 0 || timezone != 18000) {
        printf("FAILURE: zone (JST) != tzname[0] (%s) OR", tzname[0]);
        printf("timezone (18000) != (%ld)\n", timezone);
        _exit(EXIT_FAILURE);
    }
    
    try_tzset("JST");
    if (strcmp("JST", tzname[0]) != 0 || timezone != 0) {
        printf("FAILURE: zone (JST) != tzname[0] (%s) OR", tzname[0]);
        printf("timezone (0) != (%ld)\n", timezone);
        _exit(EXIT_FAILURE);
    }
    
    try_tzset("PENI-5");
    if (strcmp("PENI", tzname[0]) != 0 || timezone != -18000) {
        printf("FAILURE: zone (PENI) != tzname[0] (%s) OR", tzname[0]);
        printf("timezone (-18000) != (%ld)\n", timezone);
        _exit(EXIT_FAILURE);
    }
    
    try_tzset("PE5NI5"); // tzname[0] & [1] = '', daylight = 0, timezone = 0
    if (strcmp("", tzname[0]) != 0 || timezone != 0) {
        printf("FAILURE: zone () != tzname[0] (%s) OR", tzname[0]);
        printf("timezone (0) != (%ld)\n", timezone);
        _exit(EXIT_FAILURE);
    }

    try_tzset("CST-100");
    if (strcmp("CST", tzname[0]) != 0 || timezone != -86400) {
        printf("FAILURE: zone (CST) != tzname[0] (%s) OR", tzname[0]);
        printf("timezone (18000) != (%ld)\n", timezone);
        _exit(EXIT_FAILURE);
    }
    return 0;
}
