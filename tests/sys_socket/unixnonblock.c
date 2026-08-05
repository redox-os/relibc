#include <sys/socket.h>
#include <sys/un.h>
#include <string.h>
#include <unistd.h>
#include <stdio.h>
#include <sys/wait.h>
#include <sys/epoll.h>
#include <fcntl.h>

#include "test_helpers.h"

int main(void)
{
    int status;
    const char* socket_path = "/tmp/unix_stream.sock";
    unlink(socket_path);

    int server_fd = socket(AF_UNIX, SOCK_STREAM | SOCK_NONBLOCK, 0);
    ERROR_IF(socket, server_fd, == -1);

    struct sockaddr_un addr = { .sun_family = AF_UNIX };
    strncpy(addr.sun_path, socket_path, sizeof(addr.sun_path) - 1);

    status = bind(server_fd, (struct sockaddr*)&addr, sizeof(struct sockaddr_un));
    ERROR_IF(bind, status, == -1);

    status = listen(server_fd, 5);
    ERROR_IF(listen, status, == -1);

    pid_t pid = fork();
    ERROR_IF(fork, pid, == -1);

    if (pid == 0) {
        usleep(100000);

        int client_fd = socket(AF_UNIX, SOCK_STREAM, 0);
        ERROR_IF(socket, client_fd, == -1);

        status = connect(client_fd, (struct sockaddr*)&addr, sizeof(struct sockaddr_un));
        ERROR_IF(connect, status, == -1);

        char *msg = "ipsum";
        printf("send %s\n", msg);
        status = send(client_fd, msg, 6, 0);
        ERROR_IF(send, status, == -1);

        close(client_fd);
        return 0;
    } else {
        int epoll_fd = epoll_create1(0);
        ERROR_IF(epoll_create1, epoll_fd, == -1);

        struct epoll_event ev;
        struct epoll_event events[1];
        
        ev.events = EPOLLIN;
        ev.data.fd = server_fd;
        
        status = epoll_ctl(epoll_fd, EPOLL_CTL_ADD, server_fd, &ev);
        ERROR_IF(epoll_ctl, status, == -1);

        int nfds = epoll_wait(epoll_fd, events, 1, -1);
        UNEXP_IF(epoll_wait, nfds, <= 0); 

        int accepted_fd = -1;
        if (events[0].data.fd == server_fd) {
            accepted_fd = accept(server_fd, NULL, NULL);
            ERROR_IF(accept, accepted_fd, == -1);
        }

        close(epoll_fd);

        int flags = fcntl(accepted_fd, F_GETFL, 0);
        fcntl(accepted_fd, F_SETFL, flags & ~O_NONBLOCK);

        char x[6];
        ssize_t amount = recv(accepted_fd, x, 6, 0);
        ERROR_IF(recv, amount, == -1);

        printf("recv %s\n", x);
        close(accepted_fd);
        close(server_fd);
        
        int child_status;
        waitpid(pid, &child_status, 0);
        unlink(socket_path);
        
        if (!WIFEXITED(child_status) || WEXITSTATUS(child_status) != 0) {
            return 1;
        }
    }

    return 0;
}