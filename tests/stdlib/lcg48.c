#include <stdlib.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    long x_l, x_m;
    double x_d;
    long seedval = 0xcafebeef;
    unsigned short xsubi[3] = {0xabcd, 0xef42, 0x5678};
    
    printf("lrand48 (uninitialized):");
    for (int i = 0; i < 10; i++)
    {
        x_l = lrand48();
        printf(" %ld", x_l);
    }
    printf("\n");
    
    srand48(seedval);
    printf("drand48:");
    for (int i = 0; i < 10; i++)
    {
        x_d = drand48();
        printf(" %lf", x_d);
    }
    printf("\n");
    
    srand48(seedval);
    printf("lrand48:");
    for (int i = 0; i < 10; i++)
    {
        x_l = lrand48();
        printf(" %ld", x_l);
    }
    printf("\n");
    
    srand48(seedval);
    printf("mrand48:");
    for (int i = 0; i < 10; i++)
    {
        x_m = mrand48();
        printf(" %ld", x_m);
    }
    printf("\n");
    
    printf("erand48:");
    for (int i = 0; i < 10; i++)
    {
        x_d = erand48(xsubi);
        printf(" %lf", x_d);
    }
    printf("\n");
    
    printf("nrand48:");
    for (int i = 0; i < 10; i++)
    {
        x_l = nrand48(xsubi);
        printf(" %ld", x_l);
    }
    printf("\n");
    
    printf("jrand48:");
    for (int i = 0; i < 10; i++)
    {
        x_l = jrand48(xsubi);
        printf(" %ld", x_l);
    }
    printf("\n");
}
