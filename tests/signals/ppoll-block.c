#include <errno.h>
#include <err.h>
#include <poll.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "../test_helpers.h"
#include "signals_list.h"
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
    alarm(1); 
    struct pollfd pfd = { .fd = fds[0], .events = POLLIN };
    // POSIX requires EINTR or returning the pending events.
    int ret = ppoll(&pfd, 1, NULL, &empty);
    if ( ret < 0 )
        ERROR_IF(ppoll, ret, <0);
    if ( pfd.revents & POLLIN )
        ERROR_IF(ppoll, pfd.revents, & POLLIN);
    if ( pfd.revents & POLLOUT )
        ERROR_IF(ppoll, pfd.revents, & POLLOUT);
    if ( pfd.revents & POLLERR )
        ERROR_IF(ppoll, pfd.revents, & POLLERR);
    if ( pfd.revents & POLLHUP )
        ERROR_IF(ppoll, pfd.revents, & POLLHUP);
    if ( pfd.revents & POLLNVAL ){
        return EXIT_SUCCESS;
    }
    return 0;
}