#include <stdio.h>
#include <time.h>

int main(void) {
    /* TODO: ensure that it is really time.h supplying the NULL constant */
    printf("%p\n", NULL);

    /* Cast to long to avoid format string mismatch in case CLOCKS_PER_SEC is
    defined as some other type. The expected value (1 million) will always fit
    in a long and will always have that value on conforming systems. */
    printf("%ld\n", (long)CLOCKS_PER_SEC);
}
