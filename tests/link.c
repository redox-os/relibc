#include <stdio.h>
#include <unistd.h>

int main(int argc, char** argv) {
    link("./link.c", "./link.out");
    perror("link");
}
