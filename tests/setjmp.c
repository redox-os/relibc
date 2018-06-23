#include <stdio.h>
#include <setjmp.h>

int main() {
    jmp_buf buf;
    if (setjmp(buf)) {
        puts("hi from jump");
    } else {
        puts("jumping...");
        longjmp(buf, 0);
    }
}
