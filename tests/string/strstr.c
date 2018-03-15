#include <string.h>
#include <stdio.h>

int main(int argc, char* argv[]) {
    // should be "rust"
    char* res1 = strstr("In relibc we trust", "rust");
    printf("%s\n", (res1) ? res1 : "NULL"); 

    // should be "libc we trust"
    char* res2 = strstr("In relibc we trust", "libc");
    printf("%s\n", (res2) ? res2 : "NULL"); 

    // should be "NULL"
    char* res3 = strstr("In relibc we trust", "bugs");
    printf("%s\n", (res3) ? res3 : "NULL"); 

    return 0;
}
