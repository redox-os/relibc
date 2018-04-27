#include <time.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char** argv) {
    time_t a = 0;
    tm *time_info = gmtime(&a);

    char *time_string = asctime(time_info);

    if (time_string == NULL || strcmp(time_string, "Thu Jan  1 00:00:00 1970\n") != 0) {
        exit(1);
    }
    return 0;
}
