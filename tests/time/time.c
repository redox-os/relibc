#include <time.h>
#include <stdio.h>

int main(void) {
    struct timespec tm = {0, 0};

    int cgt = clock_gettime(CLOCK_REALTIME, &tm);
    if (cgt == -1) {
        perror("clock_gettime");
        return 1;
    }

    time_t t = time(NULL);
    if (t == (time_t)-1) {
        perror("time");
        return 1;
    }

    clock_t c = clock();
    if (c == (clock_t)-1) {
        perror("clock");
        return 1;
    }

    return 0;
}
