#include <stdlib.h>
#include <stdio.h>
#include <unistd.h>

#include "test_helpers.h"

int main(void) {
    char hostname[256] = { 0 };

    int status = gethostname(hostname, 256);
    ERROR_IF(gethostname, status, == -1);
    UNEXP_IF(gethostname, status, != 0);

    printf("Hostname: %s\n", hostname);
}
