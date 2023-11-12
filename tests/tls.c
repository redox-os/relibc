#include <stdio.h>

_Thread_local int tbss = 0;
_Thread_local int tdata = 1;

int main(void) {
    printf("%d == 0\n", tbss);
    printf("%d == 1\n", tdata);
    return 0;
}
