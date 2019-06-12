#include <stdlib.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    long x_l, x_m;
    double x_d;
    long seedval = 0xcafebeef;
    unsigned short seed[3] = {0xfedc, 0xba98, 0x7654};
    unsigned short xsubi[3] = {0xabcd, 0xef42, 0x5678};
    unsigned short lcong48_params[7] = {0x0123, 0x4567, 0x89ab, 0xcdef, 0x4242, 0xf000, 0xbaaa};
    unsigned short xsubi_from_seed48[3] = {0, 0, 0};
    
    /* Test uninitialized behavior */
    printf("lrand48 (uninitialized):");
    for (int i = 0; i < 10; i++)
    {
        x_l = lrand48();
        printf(" %ld", x_l);
    }
    printf("\n");
    
    /* Test different output types with same seed, builtin X_i and
     * default multiplier and addend */
    srand48(seedval);
    printf("drand48 (seeded with srand48):");
    for (int i = 0; i < 10; i++)
    {
        x_d = drand48();
        printf(" %lf", x_d);
    }
    printf("\n");
    
    srand48(seedval);
    printf("lrand48 (seeded with srand48):");
    for (int i = 0; i < 10; i++)
    {
        x_l = lrand48();
        printf(" %ld", x_l);
    }
    printf("\n");
    
    srand48(seedval);
    printf("mrand48 (seeded with srand48):");
    for (int i = 0; i < 10; i++)
    {
        x_m = mrand48();
        printf(" %ld", x_m);
    }
    printf("\n");
    
    /* Test corresponding functions taking user-supplied X_i, with
     * default multiplier and addend */
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
    
    /* Test seed48() "stashing" behavior.  */
    unsigned short *seed48_return = seed48(seed);
    printf("seed48_return: [%x, %x, %x]\n",
        seed48_return[0], seed48_return[1], seed48_return[2]);
    
    /* Test seeding behavior of seed48() */
    printf("lrand48 (seeded with seed48):");
    for (int i = 0; i < 10; i++)
    {
        x_l = lrand48();
        printf(" %ld", x_l);
    }
    printf("\n");
    
    /* Test restore from seed48()'s "stashed" value */
    xsubi_from_seed48[0] = seed48_return[0];
    xsubi_from_seed48[1] = seed48_return[1];
    xsubi_from_seed48[2] = seed48_return[2];
    printf("xsubi restored froom seed48 return value: [%x, %x, %x]\n",
        xsubi_from_seed48[0], xsubi_from_seed48[1], xsubi_from_seed48[2]);
    printf("nrand48 (from xsubi value returned by seed48):");
    for (int i = 0; i < 10; i++)
    {
        x_l = nrand48(xsubi_from_seed48);
        printf(" %ld", x_l);
    }
    printf("\n");
    
    /* Test behavior with all-user-defined parameters */
    lcong48(lcong48_params);
    printf("lrand48 (with parameters from lcong48):");
    for (int i = 0; i < 10; i++)
    {
        x_l = lrand48();
        printf(" %ld", x_l);
    }
    printf("\n");
    
    /* Test multiplier- and addend-restoring behavior of srand48() */
    srand48(seedval);
    printf("lrand48 (seeded with srand48 after lcong48 call):");
    for (int i = 0; i < 10; i++)
    {
        x_l = lrand48();
        printf(" %ld", x_l);
    }
    printf("\n");
    
    /* Test multiplier- and addend-restoring behavior of seed48() */
    lcong48(lcong48_params);
    seed48(seed);
    printf("lrand48 (seeded with seed48 after lcong48 call):");
    for (int i = 0; i < 10; i++)
    {
        x_l = lrand48();
        printf(" %ld", x_l);
    }
    printf("\n");
}
