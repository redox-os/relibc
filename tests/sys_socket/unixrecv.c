#include <sys/socket.h>
#include <sys/un.h>
#include <string.h>
#include <unistd.h>
#include <stdio.h>
#include <sys/wait.h>

#include "test_helpers.h"

int main(void)
{
    int status;
    const char* socket_path = "unix_stream.sock";
    unlink(socket_path);

    int server_fd = socket(AF_UNIX, SOCK_STREAM, 0);
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
        // to test that blocking in accept() works
        usleep(500000);

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
        int accepted_fd = accept(server_fd, NULL, NULL);
        ERROR_IF(accept, accepted_fd, == -1);

        char x[6];
        ssize_t amount = recv(accepted_fd, x, 6, 0);
        ERROR_IF(recv, amount, == -1);

        printf("recv %s\n", x);
        close(accepted_fd);
        close(server_fd);
        
        wait(NULL);
        unlink(socket_path);
    }

    return 0;
}