#include <assert.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void)
{
    FILE* fp = tmpfile();
    assert(fp != NULL);
    flockfile(fp);
    int c = 'c', r = 0;
    r = putc_unlocked(c, fp);
    ERROR_IF(putc_unlocked, r, == EOF);
    r = fflush(fp);
    ERROR_IF(fflush, r, == EOF);
    funlockfile(fp);
    // make sure unlock works
    r = putc(c, fp);
    ERROR_IF(putc, r, == EOF);
    r = fflush(fp);
    ERROR_IF(fflush, r, == EOF);
    return 0;
}
