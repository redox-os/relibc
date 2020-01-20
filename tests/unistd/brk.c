#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
	// sbrk report current brk
    void * current = sbrk(0);
    ERROR_IF(sbrk, current, == (void *)-1);

    // sbrk increment and report previous brk
    void * prev = current;
    current = sbrk(4096);
    ERROR_IF(sbrk, current, != prev);

    // sbrk report current break
    prev = current;
    current = sbrk(0);
    ERROR_IF(sbrk, current, != (void*)((uintptr_t)prev + 4096));

    // brk set break to new value
    int status = brk((void*)((uintptr_t)current + 4096));
    ERROR_IF(brk, status, == -1);
    UNEXP_IF(brk, status, != 0);
}
