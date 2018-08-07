#include <stdio.h>
#include <string.h>

int main() {
    char dst[20];

    strcpy(dst, "strcpy works!");
    puts(dst);
    strncpy(dst, "strncpy works!", 20);
    puts(dst);

    // Make sure no NUL is placed
    memset(dst, 'a', 20);
    dst[19] = 0;
    strncpy(dst, "strncpy should work here too", 10);
    puts(dst);
}
