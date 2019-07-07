#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    void * current = sbrk(0);
    ERROR_IF(sbrk, current, == (void *)-1);

    int status = brk(current + 4096);
    ERROR_IF(brk, status, == -1);
    UNEXP_IF(brk, status, != 0);
}
