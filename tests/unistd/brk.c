#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    int status = brk((void*)100);
    ERROR_IF(brk, status, == -1);
    UNEXP_IF(brk, status, != 0);
}
