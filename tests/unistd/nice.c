#include <unistd.h>
#include <sys/resource.h>
#include <errno.h>
#include "test_helpers.h"

int main(void) {
    // XXX: in linux sometimes the start prio is random
    int s1 = setpriority(PRIO_PROCESS, 0, 0);
    ERROR_IF(setpriority, s1, == -1);

    errno = 0;
    int p1 = getpriority(PRIO_PROCESS, 0);
    ERROR_IF(getpriority, errno, != 0);

    errno = 0;
    int n1 = nice(5);
    ERROR_IF(nice, errno, != 0);

    errno = 0;
    int p2 = getpriority(PRIO_PROCESS, 0);
    ERROR_IF(getpriority, errno, != 0);
    UNEXP_IF(nice, p2, != p1 + 5);
    UNEXP_IF(nice, n1, != p2);

    int s2 = setpriority(PRIO_PROCESS, 0, 15);
    ERROR_IF(setpriority, s2, == -1);

    errno = 0;
    int p3 = getpriority(PRIO_PROCESS, 0);
    ERROR_IF(getpriority, errno, != 0);
    UNEXP_IF(getpriority, p3, != 15);

    return 0;
}