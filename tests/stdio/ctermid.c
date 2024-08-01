#include <stdio.h>
#include <string.h>

#define DEVTTY "/dev/tty"

int main(void)
{
    char *name = ctermid(NULL);
    if(strcmp(name, DEVTTY) != 0) {
        printf("ctermid name differs: expected %s, got: %s\n", DEVTTY, name);
        return 1;
    }

    char name2[L_ctermid];
    ctermid(name2);
    if(strcmp(name, DEVTTY) != 0) {
        printf("ctermid name2 differs: expected %s, got: %s\n", DEVTTY, name2);
        return 1;
    }

    return 0;
}