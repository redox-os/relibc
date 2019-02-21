#include <stdlib.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    printf("%d\n", rand());
    srand(259);
    printf("%d\n", rand());
}
