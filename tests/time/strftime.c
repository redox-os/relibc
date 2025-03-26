#include <stdlib.h>
#include <stdio.h>
#include <time.h>

#include "test_helpers.h"

#define PRINT_BUFSZ 50
#define V_BUFSZ 3

void print(time_t timestamp, char* fmt) {
    char out[PRINT_BUFSZ] = {0};
    size_t n = strftime(out, PRINT_BUFSZ, fmt, localtime(&timestamp));
    printf("%zu: %s\n", n, out);
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

    return EXIT_SUCCESS;
}
