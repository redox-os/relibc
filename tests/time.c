#include <time.h>
#include <stdio.h>

int main(int argc, char** argv) {
    timespec tm = {0, 0};
    clock_gettime(CLOCK_REALTIME, &tm);
    perror("clock_gettime");
    time(NULL);
    perror("time");
    clock_t c = clock();
    perror("clock");
    return 0;
}
