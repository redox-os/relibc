#include <stdio.h>
#include <stdlib.h>
#include <sys/epoll.h>
#include <unistd.h>
#include <fcntl.h>

#include "../test_helpers.h"

struct Context {
    int dummy;
};

int main() {
    int epfd = epoll_create1(EPOLL_CLOEXEC);
    ERROR_IF(epoll_create1, epfd, == -1);
    int pipefd[2];
    pipe(pipefd);

    struct Context *ctx = malloc(sizeof(struct Context));
    ctx->dummy = 1;

    struct epoll_event ev;
    ev.events = EPOLLIN;
    ev.data.ptr = ctx;

    int status = epoll_ctl(epfd, EPOLL_CTL_ADD, pipefd[0], &ev);
    ERROR_IF(epoll_ctl, status, == -1);

    struct epoll_event del_ev;
    del_ev.data.ptr = NULL;
    status = epoll_ctl(epfd, EPOLL_CTL_DEL, pipefd[0], &del_ev);
    ERROR_IF(epoll_ctl, status, == -1);
    free(ctx);
    ctx = NULL;

    write(pipefd[1], "data", 4);
    write(pipefd[1], "data", 4);

    struct epoll_event events[1];
    int evs = 0;
    while (1) {
        int nfds = epoll_wait(epfd, events, 1, 1000);
        evs += nfds;
        if (nfds > 0) {
            printf("garbage data: %p\n", events[1].data.ptr); 
        } else {
            break;
        }
    }

    UNEXP_IF(epoll_wait, evs, != 0);
    return 0;
}