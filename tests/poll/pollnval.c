#include <fcntl.h>
#include <stdio.h>
#include <poll.h>
#include <sys/wait.h>
#include <unistd.h>
#include <assert.h>
#include <errno.h>

#include "../test_helpers.h"

int main() {
    struct pollfd fds[2];
    
    fds[0].fd = -1; // ignored
    fds[0].events = POLLIN;
    fds[0].revents = 0;

    fds[1].fd = 9999; // POLLNVAL
    fds[1].events = POLLIN;
    fds[1].revents = 0;
    
    int nfds = poll(fds, 2, 0);
    
    if (nfds < 0) {
        perror("poll");
        return 1;
    }
    
    UNEXP_IF("poll", nfds, != 1);
    UNEXP_IF("poll", fds[0].revents, != 0);
    UNEXP_IF("poll", (fds[1].revents & POLLNVAL), == 0);
    
    return 0;
}
