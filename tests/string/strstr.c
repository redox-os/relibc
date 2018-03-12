#include <string.h>
#include <stdio.h>

int main(int argc, char* argv[]) {
    // should be "rust"
    char* str1 = strstr("In relibc we trust", "rust");
    printf("%s\n", (str1) ? str1 : "NULL"); 

    // should be "libc we trust"
    char* str2 = strstr("In relibc we trust", "libc");
    printf("%s\n", (str2) ? str2 : "NULL"); 

    // should be "NULL"
    char* str3 = strstr("In relibc we trust", "bugs");
    printf("%s\n", (str3) ? str3 : "NULL"); 

    return 0;
}
