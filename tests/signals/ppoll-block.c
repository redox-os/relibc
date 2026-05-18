
#include <stdio.h>
#include <stdlib.h>
#include "../test_helpers.h"
#include "signals_list.h"
#include <errno.h>
#include <poll.h>
#include <unistd.h>
#include <stdio.h>
#include <err.h>
#include <fcntl.h>
#include "signal.h"
#include <assert.h>
#include <sys/wait.h>
#include <signal.h>
#include <stdio.h>
#include <pthread.h>
#include <unistd.h>
#include <limits.h>
#include <errno.h>
#include <sys/utsname.h>

#include "test_helpers.h"
static void handler(int signum)
{
    (void) signum;
    int errnum = errno;
    printf("SIGUSR1\n");
    fflush(stdout);
    errno = errnum;
}

int main(void)
{
    signal(SIGUSR1, handler);
    sigset_t sigusr1;
    sigemptyset(&sigusr1);
    sigaddset(&sigusr1, SIGUSR1);
    sigprocmask(SIG_BLOCK, &sigusr1, NULL);
    sigset_t empty;
    sigemptyset(&empty);
    int fds[2];
    if ( pipe(fds) )
        err(1, "pipe");
    close(fds[0]);
    printf("Hi, my FD is %d\n",fds[0]);
    //long fcn_ret = fcntl(fds[0], F_GETFD, 0) ;

    //printf("fcn_ret is %d\n, errno is %d\n",fcn_ret, errno);
    errno = 0;

    alarm(1);
    struct pollfd pfd = { .fd = fds[0], .events = POLLIN };
    // POSIX requires EINTR or returning the pending events.
    int ret = ppoll(&pfd, 1, NULL, &empty);
    printf("ppoll ret: %d, errno: %d\n, events %d\n", ret, errno, pfd.revents);
    if ( pfd.revents & POLLIN ){
        printf("HI POLLLIN\n");
    }
    if ( ret < 0 )
        ERROR_IF(ppoll, ret, <0);
    //if ( !ret )
    //{
    //    printf("ppoll() == 0\n");
    //    return 0;
    //}
    printf("0");
    if ( pfd.revents & POLLIN ){
        ERROR_IF(ppoll, pfd.revents, & POLLIN);
    }
    if ( pfd.revents & POLLOUT )
        ERROR_IF(ppoll, pfd.revents, & POLLOUT);
    if ( pfd.revents & POLLERR )
        ERROR_IF(ppoll, pfd.revents, & POLLERR);
    if ( pfd.revents & POLLHUP )
        ERROR_IF(ppoll, pfd.revents, & POLLHUP);
    if ( pfd.revents & POLLNVAL ){
        printf("\nPOLLNVAL");
        return EXIT_SUCCESS;
    }
    printf("\n");
    return 0;
}