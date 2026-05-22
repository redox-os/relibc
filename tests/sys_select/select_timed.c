#include <sys/select.h>

#include <string.h>
#include <stdio.h>
#include <unistd.h>

#include "test_helpers.h"

int main(void)
{
    int status;
    int pipe_fds[2];
    
    status = pipe(pipe_fds);
    ERROR_IF(pipe, status, == -1);

    int read_fd  = pipe_fds[0];
    int write_fd = pipe_fds[1];

    fd_set read_fds;
    FD_ZERO(&read_fds);
    FD_SET(read_fd, &read_fds);

    struct timeval timeout = { .tv_sec = 0, .tv_usec = 10000 };
    
    int ready = select(read_fd + 1, &read_fds, NULL, NULL, &timeout);
    ERROR_IF(pselect, ready, == -1);
    UNEXP_IF(pselect, ready, != 0);

    char *c = "foo";
    ssize_t written = write(write_fd, c, 4);
    ERROR_IF(write, written, == -1);
    UNEXP_IF(write, written, != 4);

    FD_ZERO(&read_fds);
    FD_SET(read_fd, &read_fds);
    
    timeout.tv_sec = 5;
    timeout.tv_usec = 0;

    ready = select(read_fd + 1, &read_fds, NULL, NULL, &timeout);
    ERROR_IF(pselect, ready, == -1);
    UNEXP_IF(pselect, ready, != 1);

    int is_set = FD_ISSET(read_fd, &read_fds);
    UNEXP_IF(FD_ISSET, is_set, == 0);

    char x[4];
    ssize_t amount = read(read_fd, x, 4);
    ERROR_IF(read, amount, == -1);
    UNEXP_IF(read, amount, != 4);

    status = strcmp(c, x);
    printf("write %s\n", c);
    printf("read  %s\n", x);
    UNEXP_IF(strcmp, status, != 0);

    close(read_fd);
    close(write_fd);

    return 0;
}