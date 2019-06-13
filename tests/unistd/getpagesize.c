#include <stdio.h>
#include <unistd.h>
#include <errno.h>

#include "test_helpers.h"

int main(void) {
    errno = 0;
    int getpagesize_result = getpagesize();
    int getpagesize_errno = errno;
    
    printf("getpagesize(): %d, errno: %d = %s\n", getpagesize_result,
        getpagesize_errno, strerror(getpagesize_errno));
}
