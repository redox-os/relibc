/*
 * The process joins process group 0.
 */
#include <stdio.h>
#include <sys/types.h>
#include <unistd.h>

#include "test_helpers.h"

int main(void) {
    int pg_status = setpgid(getpid(), 0);
    ERROR_IF(setpgid, pg_status, == -1);
    UNEXP_IF(setpgid, pg_status, != 0);

    printf("%d belongs to process group %d\n", getpid(), getpgrp());

    int reg_status = setregid(-1, -1);
    ERROR_IF(setregid, reg_status, == -1);
    UNEXP_IF(setregid, reg_status, != 0);

    printf("%d has egid %d and gid %d\n", getpid(), getegid(), getgid());

    int reu_status = setreuid(-1, -1);
    ERROR_IF(setreuid, reu_status, == -1);
    UNEXP_IF(setreuid, reu_status, != 0);

    printf("%d has euid %d and uid %d\n", getpid(), geteuid(), getuid());
}
