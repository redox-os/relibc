#include <errno.h>
#include <stdio.h>
#include <unistd.h>

#include "test_helpers.h"

#define PC(N) \
    do { \
        errno = 0; \
        printf("%s (%d): %ld (%d)\n", #N, _PC_ ## N, fpathconf(0, _PC_ ## N), errno); \
    } while (0)

int main(void) {
    PC(LINK_MAX);
    PC(MAX_CANON);
    PC(MAX_INPUT);
    PC(NAME_MAX);
    PC(PATH_MAX);
    PC(PIPE_BUF);
    PC(CHOWN_RESTRICTED);
    PC(NO_TRUNC);
    PC(VDISABLE);
    PC(SYNC_IO);
    PC(ASYNC_IO);
    PC(PRIO_IO);
    PC(SOCK_MAXBUF);
    PC(FILESIZEBITS);
    PC(REC_INCR_XFER_SIZE);
    PC(REC_MAX_XFER_SIZE);
    PC(REC_MIN_XFER_SIZE);
    PC(REC_XFER_ALIGN);
    PC(ALLOC_SIZE_MIN);
    PC(SYMLINK_MAX);
    PC(2_SYMLINKS);
}
