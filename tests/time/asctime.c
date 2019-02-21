#include <time.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    time_t a = 0;
    struct tm *time_info = gmtime(&a);

    char *time_string = asctime(time_info);

    if (time_string == NULL || strcmp(time_string, "Thu Jan  1 00:00:00 1970\n") != 0) {
        exit(EXIT_FAILURE);
    }
}
