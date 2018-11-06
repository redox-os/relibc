#include <math.h>
#include <stdio.h>

int main(int argc, char ** argv) {
    // Check the existence of math constants
    printf("M_E = %.4f\n", M_E);
    printf("M_LOG2E = %.4f\n", M_LOG2E);
    printf("M_LOG10E = %.4f\n", M_LOG10E);
    printf("M_LN2 = %.4f\n", M_LN2);
    printf("M_LN10 = %.4f\n", M_LN10);
    printf("M_PI = %.4f\n", M_PI);
    printf("M_PI_2 = %.4f\n", M_PI_2);
    printf("M_PI_4 = %.4f\n", M_PI_4);
    printf("M_1_PI = %.4f\n", M_1_PI);
    printf("M_2_PI = %.4f\n", M_2_PI);
    printf("M_2_SQRTPI = %.4f\n", M_2_SQRTPI);
    printf("M_SQRT2 = %.4f\n", M_SQRT2);
    printf("M_SQRT1_2 = %.4f\n", M_SQRT1_2);

    double pi = 3.14;
    float c = cos(pi);
    printf("cos(%f) = %f\n", pi, c);
    return 0;
}
