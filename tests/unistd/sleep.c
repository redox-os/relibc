#include <time.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    sleep(2);
    perror("sleep");
    usleep(1000);
    perror("usleep");
    struct timespec tm = {0, 10000};
    nanosleep(&tm, NULL);
    perror("nanosleep");
    return EXIT_SUCCESS;
}
