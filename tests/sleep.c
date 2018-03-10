#include <time.h>
#include <unistd.h>
#include <stdio.h>

int main(int argc, char** argv) {
    sleep(2);
    perror("sleep");
    usleep(1000);
    perror("usleep");
    timespec tm = {0, 10000};
    nanosleep(&tm, NULL);
    perror("nanosleep");
}
