#include <string.h>
#include <stdio.h>

int main(int argc, char* argv[]) {
    char* str = "The quick drawn fix jumps over the lazy bug";

    // should be "The quick drawn fix jumps over the lazy bug"
    char* str1 = strpbrk(str, "From The Very Beginning");
    printf("%s\n", (str1) ? str1 : "NULL"); 

    // should be "lazy bug"
    char* str2 = strpbrk(str, "lzbg");
    printf("%s\n", (str2) ? str2 : "NULL"); 


    // should be "NULL"
    char* str3 = strpbrk(str, "404");
    printf("%s\n", (str3) ? str3 : "NULL"); 

    return 0;
}
