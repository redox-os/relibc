#include <stdlib.h>
#include <stdio.h>
#include <math.h> // INFINITY, NAN constants

#include "test_helpers.h"

int main(void) {
    /* Includes extrema for the double type, subnormal values and other
     * corner cases. */
    double test_values[] = {0.0, 0.5, 1.0, 42, \
        -0.0, -0.5, -1.0, -42, \
        3.14e+15, 1.337e+20, \
        -3.14e+15, -1.337e+20, \
        -1.7976931348623157e+308, \
        4.9406564584124654e-324, \
        2.2250738585072014e-308, \
        1.7976931348623157e+308, \
        1.0000000000000002, \
        INFINITY, -INFINITY, NAN, -NAN};
    
    // Test near zero and near buffer size in particular
    int test_ndigit_values[] = {-2, -1, 0, 1, 2, 3, 6, 14, 15, 16, 17, 18, 19, 100};
    
    for (size_t i = 0; i < sizeof(test_values)/sizeof(double); i++) {
        for (size_t j = 0; j < sizeof(test_ndigit_values)/sizeof(int); j++) {
            double value = test_values[i];
            int ndigit = test_ndigit_values[j];
            
            int decpt = 0;
            int sign = 0;
            
            char *ecvt_return = NULL;
            
            /* TODO: apparently these values are a stress test for
             * printf as well... */
            //printf("ecvt (value = %le, ndigit = %d): ", value, ndigit);
            printf("ecvt (ndigit = %d):", ndigit);
            
            ecvt_return = ecvt(value, ndigit, &decpt, &sign);
            
            printf(" decpt: %d,", decpt);
            printf(" sign: %d,", sign);
            printf(" returned: %s\n", ecvt_return);
        }
    }
}
