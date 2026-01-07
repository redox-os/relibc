#include <arpa/inet.h>

#include "test_helpers.h"

int main(void)
{
    int status;
    int listen_fd = socket(AF_INET, SOCK_STREAM, 0);
    ERROR_IF(socket, listen_fd, == -1);

    struct sockaddr_in addr =
    {
        .sin_family = AF_INET,
        .sin_addr = { .s_addr = htonl(0x7F000001 /* 127.0.0.1 */) },
        .sin_port = htons(0),
    };
    struct sockaddr* saddr = (struct sockaddr*)&addr;
    socklen_t addr_len = sizeof(struct sockaddr_in);
    status = bind(listen_fd, saddr, addr_len);
    ERROR_IF(bind, status, == -1);

    status = listen(listen_fd, 1);
    ERROR_IF(listen, status, == -1);

    status = getsockname(listen_fd, saddr, &addr_len);
    ERROR_IF(getsockname, status, == -1);

    int client_fd = socket(AF_INET, SOCK_STREAM, 0);
    ERROR_IF(socket, client_fd, == -1);

    status = connect(client_fd, saddr, addr_len);
    ERROR_IF(connect, status, == -1);

    int server_fd = accept(listen_fd, NULL, NULL);
    ERROR_IF(accept, status, == -1);

    char *c = "foo";
    status = send(server_fd, c, 4, 0);
    ERROR_IF(send, status, == -1);

    char x[4];
    ssize_t amount = recv(client_fd, x, 4, 0);
    ERROR_IF(recv, amount, == -1);
    UNEXP_IF(recv, amount, != 4);

    status = strcmp(c, x);
    printf("send %s\n", c);
    printf("recv %s\n", x);
    UNEXP_IF(strcmp, status, != 0);
}
