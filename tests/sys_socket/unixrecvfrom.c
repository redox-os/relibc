#include <sys/socket.h>
#include <sys/un.h>
#include <string.h>
#include <unistd.h>

#include "test_helpers.h"

int main(void)
{
    int status;
    const char* socket_path = "unixrecvfrom.sock";
    
    unlink(socket_path);

    int server_fd = socket(AF_UNIX, SOCK_DGRAM, 0);
    ERROR_IF(socket, server_fd, == -1);

    struct sockaddr_un addr = { .sun_family = AF_UNIX };
    strncpy(addr.sun_path, socket_path, sizeof(addr.sun_path) - 1);

    status = bind(server_fd, (struct sockaddr*)&addr, sizeof(struct sockaddr_un));
    ERROR_IF(bind, status, == -1);

    int client_fd = socket(AF_UNIX, SOCK_DGRAM, 0);
    ERROR_IF(socket, client_fd, == -1);
    
    status = connect(client_fd, (struct sockaddr*)&addr, sizeof(struct sockaddr_un));
    ERROR_IF(connect, status, == -1);

    char *c = "lorem";
    status = send(client_fd, c, 6, 0);
    ERROR_IF(send, status, == -1);

    char x[6];
    struct sockaddr_un from;
    socklen_t from_len = sizeof(struct sockaddr_un);
    
    ssize_t amount = recvfrom(server_fd, x, 6, 0, (struct sockaddr*)&from, &from_len);
    ERROR_IF(recvfrom, amount, == -1);
    UNEXP_IF(recvfrom, amount, != 6);

    status = strcmp(c, x);
    printf("send     %s\n", c);
    printf("recvfrom %s\n", x);
    UNEXP_IF(strcmp, status, != 0);

    close(server_fd);
    close(client_fd);
    unlink(socket_path);
}