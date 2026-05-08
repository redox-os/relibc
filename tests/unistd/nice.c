#include <unistd.h>
#include <sys/resource.h>
#include <errno.h>
#include "test_helpers.h"

int main(void) {
    errno = 0;
    int p1 = getpriority(PRIO_PROCESS, 0);
    ERROR_IF(getpriority, errno, != 0);

    errno = 0;
    int n1 = nice(5);
    ERROR_IF(nice, errno, != 0);

    errno = 0;
    int p2 = getpriority(PRIO_PROCESS, 0);
    ERROR_IF(getpriority, errno, != 0);
    // XXX: in linux sometimes the start prio is random
    UNEXP_IF(nice, p2, != p1 + 5 && p2 != 19);
    UNEXP_IF(nice, n1, != p2);

    return 0;
}