#include <locale.h>
#include <stdio.h>
#include <string.h>

int main() {
    // TODO: Implement locale properly and test it here
    char* val = setlocale(LC_ALL, NULL);
    if (strcmp(val, "C") == 0) {
        puts("success!");
    } else {
        printf("setlocale returned the wrong value: %s", val);
    }
}
