#include <time.h>
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    struct timespec tm = {0, 0};

    int cgt = clock_gettime(CLOCK_REALTIME, &tm);
    if (cgt == -1) {
        perror("clock_gettime");
        exit(EXIT_FAILURE);
    }

    time_t t = time(NULL);
    if (t == (time_t)-1) {
        perror("time");
        exit(EXIT_FAILURE);
    }

    clock_t c = clock();
    if (c == (clock_t)-1) {
        perror("clock");
        exit(EXIT_FAILURE);
    }
}
