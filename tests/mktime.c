#include <stdio.h>
#include <time.h>

int main() {
    struct tm t = {};

    t.tm_year = 71;
    t.tm_mday = 1;

    printf("%ld\n", mktime(&t));

    int day = 60 * 60 * 24;
    time_t inputs[] = { -(day * 33), -day, -500, 0, 1531454950 };
    for (int i = 0; i < 5; i += 1) {
        struct tm* t2 = localtime(&inputs[i]);

        printf("%ld = %ld\n", inputs[i], mktime(t2));
        if (inputs[i] != mktime(t2)) {
            puts("Failed!");
        }
    }
}
