#include <unistd.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    gid_t egid = getegid();
    uid_t euid = geteuid();
    gid_t gid = getgid();
    pid_t pgid = getpgid(0);
    pid_t pid = getpid();
    pid_t ppid = getppid();
    uid_t uid = getuid();
    printf("egid: %d, euid: %d, gid: %d, pgid: %d, pid: %d, ppid %d, uid %d\n",
            egid, euid, gid, pgid, pid, ppid, uid);
}
