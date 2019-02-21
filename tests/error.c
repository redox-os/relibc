#include <unistd.h>
#include <stdio.h>
#include <string.h>
#include <errno.h>

#include "test_helpers.h"

int main(void) {
    chdir("nonexistent");
    printf("errno: %d = %s\n", errno, strerror(errno));
    perror("perror");
}
