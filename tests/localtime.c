#include <stdio.h>
#include <time.h>

int main() {
    int day = 60 * 60 * 24;
    time_t inputs[] = { -(day * 33), -day, -500, 0, 1531454950 };
    for (int i = 0; i < 5; i += 1) {
        struct tm* t = localtime(&inputs[i]);

        printf(
            "Year %d, Day of year: %d, Month %d, Day of month: %d, Day of week: %d, %d:%d:%d\n",
            t->tm_year, t->tm_yday, t->tm_mon, t->tm_mday, t->tm_wday, t->tm_hour, t->tm_min, t->tm_sec
        );
    }

    time_t input = 1531461823;
    fputs(ctime(&input), stdout); // Omit newline
}
