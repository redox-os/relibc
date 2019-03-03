#include <stdio.h>
#include <setjmp.h>

#include "test_helpers.h"

int main(void) {
    jmp_buf buf;
    if (setjmp(buf)) {
        puts("hi from jump");
    } else {
        puts("jumping...");
        longjmp(buf, 0);
    }
}
