#include <stdlib.h>
#include <stdio.h>

int main(void) {
    printf("%d\n", rand());
    srand(259);
    printf("%d\n", rand());
}
