#include <math.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    double pi = 3.14;
    float c = cos(pi);
    printf("cos(%f) = %f\n", pi, c);
}
