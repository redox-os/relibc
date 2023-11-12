#include <fcntl.h>
#include <stdio.h>
#include <sys/epoll.h>
#include <sys/wait.h>
#include <unistd.h>
#include <errno.h>

#include "../test_helpers.h"

int reader(int fd) {
    // Create an epoll file
    int epollfd = epoll_create1(EPOLL_CLOEXEC);
    if (epollfd < 0) {
        perror("epoll_create1");
        return 1;
    }

    // Register for events from the reader file
    struct epoll_event ev;
    ev.events = EPOLLIN;
    ev.data.fd = fd;
    if (epoll_ctl(epollfd, EPOLL_CTL_ADD, fd, &ev) < 0) {
        perror("epoll_ctl");
        return 1;
    }

    struct epoll_event events[8];

    // Check that epoll returns error on a zero or negative number of events
    int nfds0 = epoll_wait(epollfd, events, 0, -1);
    if (nfds0 != -1 || errno != EINVAL) {
        perror("epoll_wait");
        return 1;
    }

    int nfds_n1 = epoll_wait(epollfd, events, -1, -1);
    if (nfds_n1 != -1 || errno != EINVAL) {
        perror("epoll_wait");
        return 1;
    }

    // Process exactly 1024 events
    for (int i = 0; i < 1024; i++) {
        // Wait for the next event
        int nfds = epoll_wait(epollfd, events, sizeof(events)/sizeof(struct epoll_event), -1);
        if (nfds < 0) {
            perror("epoll_wait");
            return 1;
        }

        // For each event received
        for (int n = 0; n < nfds; n++) {
            // If the event is the reader file
            if (events[n].data.fd == fd) {
                // Read the current event count
                int writer_i;
                ssize_t status = read(fd, &writer_i, sizeof(writer_i));
                ERROR_IF(read, status, == -1);
                size_t count = (size_t)status;

                if (count < sizeof(writer_i)) {
                    fprintf(stderr, "read %zu instead of %d\n", count, sizeof(writer_i));
                    return 1;
                }
                // Make sure the writer's event count matches our own
                if (i != writer_i) {
                    fprintf(stderr, "received event count %d instead of %d\n", writer_i, i);
                    return 1;
                }
                printf("%d == %d\n", i, writer_i);
            } else {
                // Otherwise, return an error
                fprintf(stderr, "unknown fd %d\n", events[n].data.fd);
                return 1;
            }
        }
    }

    return 0;
}

int writer(int fd) {
    // Create an epoll file
    int epollfd = epoll_create1(EPOLL_CLOEXEC);
    if (epollfd < 0) {
        perror("epoll_create1");
        return 1;
    }

    // Register for events from the writer file
    struct epoll_event ev;
    ev.events = EPOLLOUT;
    ev.data.fd = fd;
    if (epoll_ctl(epollfd, EPOLL_CTL_ADD, fd, &ev) < 0) {
        perror("epoll_ctl");
        return 1;
    }

    // Process exactly 1024 events
    struct epoll_event events[8];
    for (int i = 0; i < 1024; i++) {
        // Wait for the next event
        int nfds = epoll_wait(epollfd, events, sizeof(events)/sizeof(struct epoll_event), -1);
        if (nfds < 0) {
            perror("epoll_wait");
            return 1;
        }

        // For each event received
        for (int n = 0; n < nfds; n++) {
            // If the event is the writer file
            if (events[n].data.fd == fd) {
                // Write the current event count
                ssize_t status = write(fd, &i, sizeof(i));
                ERROR_IF(write, status, == -1);
                size_t count = (size_t)status;

                if (count < sizeof(i)) {
                    fprintf(stderr, "wrote %zu instead of %d\n", count, sizeof(i));
                    return 1;
                }
            } else {
                // Otherwise, return an error
                fprintf(stderr, "unknown fd %d\n", events[n].data.fd);
                return 1;
            }
        }
    }

    return 0;
}

int main(void) {
    // Create a non-blocking pipe to use for epoll testing
    int pipefd[2];
    if (pipe2(pipefd, O_CLOEXEC | O_NONBLOCK) < 0) {
        perror("pipe2");
        return 1;
    }

    pid_t pid = fork();
    if (pid < 0) {
        perror("fork");
        return 1;
    } else if (pid == 0) {
        // Child process will read events
        close(pipefd[1]);
        return reader(pipefd[0]);
    } else {
        // Parent process will write events
        close(pipefd[0]);
        int ret = writer(pipefd[1]);

        // Wait for child process
        int status = 0;
        if (waitpid(pid, &status, 0) != pid) {
            perror("waitpid");
            return 1;
        }

        // If writer failed, return exit status
        if (ret != 0) {
            return ret;
        }

        // If child exited with exit status
        if (WIFEXITED(status)) {
            // Return the child's exit status
            return WEXITSTATUS(status);
        } else {
            // Otherwise, return 1
            return 1;
        }
    }
}
