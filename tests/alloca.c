#include <alloca.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    char *str = (char *) alloca(17);

    memset(str, 'A', 16);
    str[16] = '\0';

    printf("%s\n", str);
}
