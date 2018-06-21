#include <stdio.h>
#include <malloc.h>

int main(int argc, char ** argv) {
    int a = 0;
    int b = 0;
    int val = sscanf("123 0x321", "%d %i", &a, &b);
    if (val != 2) {
        printf("error: %d\n", val);
    } else {
        if (a == 123 && b == 0x321) {
            puts("success!");
        } else {
            printf("incorrect results: { a: %d, b: %x }\n", a, b);
        }
    }
}
