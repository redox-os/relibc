#include <time.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern int daylight;
extern long timezone;
extern const char *tzname[2];
void tzset(void);

int main(void) {
    tzset();
    printf("tzname[0] %s, tzname[1] %s, daylight %d, timezone %ld\n",
        tzname[0], tzname[1], daylight, timezone);
    return 0;
}
