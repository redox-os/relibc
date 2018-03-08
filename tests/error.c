#include <unistd.h>
#include <stdio.h>
#include <errno.h>

int main(int argc, char** argv) {
    chdir("nonexistent"); 
    printf("errno: %d\n", errno);
}
