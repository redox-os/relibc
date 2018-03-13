#include <unistd.h>
#include <stdio.h>

int main(int argc, char** argv) {
    link("./unlink.c", "./unlink.out");
    perror("unlink");
    return 0;
}
