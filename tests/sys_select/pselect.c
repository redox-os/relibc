#include <sys/select.h>
#include <signal.h>

#include <string.h>
#include <stdio.h>
#include <unistd.h>

#include "test_helpers.h"

volatile sig_atomic_t got_signal = 0;

void handle_sigusr1(int sig)
{
    (void)sig;
    got_signal = 1;
}

int main(void)
{
    int status;
    int pipe_fds[2];
    
    status = pipe(pipe_fds);
    ERROR_IF(pipe, status, == -1);

    int read_fd  = pipe_fds[0];
    int write_fd = pipe_fds[1];

    // test sigmask
    struct sigaction sa;
    memset(&sa, 0, sizeof(sa));
    sa.sa_handler = handle_sigusr1;
    status = sigaction(SIGUSR1, &sa, NULL);
    ERROR_IF(sigaction, status, == -1);

    sigset_t block_mask, orig_mask;
    sigemptyset(&block_mask);
    sigaddset(&block_mask, SIGUSR1); // block SIGUSR1
    status = sigprocmask(SIG_BLOCK, &block_mask, &orig_mask);
    ERROR_IF(sigprocmask, status, == -1);

    status = raise(SIGUSR1);
    ERROR_IF(raise, status, == -1);
    UNEXP_IF(raise, got_signal, != 0); // must not fired

    fd_set read_fds;
    FD_ZERO(&read_fds);
    FD_SET(read_fd, &read_fds);

    struct timespec timeout = { .tv_sec = 5, .tv_nsec = 0 };

    int ready = pselect(read_fd + 1, &read_fds, NULL, NULL, &timeout, &orig_mask);
    
    UNEXP_IF(pselect, ready, != -1); // errno should be EINTR
    UNEXP_IF(pselect_signal_handler, got_signal, != 1); // must been fired

    status = sigprocmask(SIG_SETMASK, &orig_mask, NULL);
    ERROR_IF(sigprocmask, status, == -1);

    // test timespec
    char *c = "bar";
    ssize_t written = write(write_fd, c, 4);
    ERROR_IF(write, written, == -1);
    UNEXP_IF(write, written, != 4);

    FD_ZERO(&read_fds);
    FD_SET(read_fd, &read_fds);
    
    timeout.tv_sec = 999;
    timeout.tv_nsec = 0;

    ready = pselect(read_fd + 1, &read_fds, NULL, NULL, &timeout, NULL);
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