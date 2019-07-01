#include <stdio.h>

_Thread_local int tbss = 0;
_Thread_local int tdata = 1;

int main(int argc, char ** argv) {
    printf("%d == 0\n", tbss);
    printf("%d == 1\n", tdata);
    return 0;
}
