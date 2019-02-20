#include <unistd.h>
#include <sys/stat.h>
#include <stdio.h>

int main(void) {
    mkdir("foo", 0);
    int status = rmdir("foo");
    printf("rmdir exited with status code %d\n", status);
    return 0;
}
