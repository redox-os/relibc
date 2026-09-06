#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <unistd.h>

#include "test_helpers.h"

int main(void) {
    {
        int fd = creat("symlink_target", 0777);
        ERROR_IF(creat, fd, == -1);
        UNEXP_IF(creat, fd, < 0);

        int c1 = close(fd);
        ERROR_IF(close, c1, == -1);
        UNEXP_IF(close, c1, != 0);
    }

    // symlink to existing target
    int s1 = symlink("symlink_target", "symlink_link");
    ERROR_IF(symlink, s1, == -1);
    UNEXP_IF(symlink, s1, != 0);

    // symlink and target exist
    int s2 = symlink("symlink_target", "symlink_link");
    UNEXP_IF(symlink, s2, != -1);
    UNEXP_IF(symlink_overwrite_errno, errno, != EEXIST);

    // TODO: why O_DIRECTORY don't work on linux?
    int dirfd = open("/tmp", O_RDONLY); // | O_DIRECTORY
    ERROR_IF(open, dirfd, == -1);
    UNEXP_IF(open, dirfd, < 0);

    // symlink to void target
    int s3 = symlinkat("symlink_target", dirfd, "symlink_link_at");
    ERROR_IF(symlinkat, s3, == -1);
    UNEXP_IF(symlinkat, s3, != 0);


    // symlink exist target void
    int s4 = symlinkat("symlink_target", dirfd, "symlink_link_at");
    UNEXP_IF(symlinkat, s4, != -1);
    UNEXP_IF(symlinkat_overwrite_errno, errno, != EEXIST);

    int c2 = close(dirfd);
    ERROR_IF(close, c2, == -1);

    int u1 = unlink("symlink_link");
    ERROR_IF(unlink, u1, == -1);

    int u2 = unlink("/tmp/symlink_link_at");
    ERROR_IF(unlink, u2, == -1);

    int u3 = unlink("symlink_target");
    ERROR_IF(unlink, u3, == -1);

    return 0;
}
