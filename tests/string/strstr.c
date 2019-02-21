#include <string.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    printf("%s\n", strstr("In relibc we trust", "rust"));
    printf("%s\n", strstr("In relibc we trust", "libc"));
    printf("%s\n", strstr("In relibc we trust", "bugs"));
    printf("%s\n", strstr("IN RELIBC WE TRUST", "rust"));
    printf("%s\n", strcasestr("IN RELIBC WE TRUST", "rust"));
}
