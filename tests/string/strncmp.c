#include <string.h>
#include <stdio.h>

int main(int argc, char* argv[]) {
    printf("%d\n", strncmp("a", "aa", 2));
    printf("%d\n", strncmp("a", "aä", 2));
    printf("%d\n", strncmp("\xFF", "\xFE", 2));
    printf("%d\n", strncmp("", "\xFF", 1));
    printf("%d\n", strncmp("a", "c", 1));
    printf("%d\n", strncmp("a", "a", 2));

    puts("test");

    return 0;
}
