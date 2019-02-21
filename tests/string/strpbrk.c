#include <string.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    char* source = "The quick drawn fix jumps over the lazy bug";

    // should be "The quick drawn fix jumps over the lazy bug"
    char* res1 = strpbrk(source, "From The Very Beginning");
    printf("%s\n", (res1) ? res1 : "NULL"); 

    // should be "lazy bug"
    char* res2 = strpbrk(source, "lzbg");
    printf("%s\n", (res2) ? res2 : "NULL"); 

    // should be "NULL"
    char* res3 = strpbrk(source, "404");
    printf("%s\n", (res3) ? res3 : "NULL");
}
