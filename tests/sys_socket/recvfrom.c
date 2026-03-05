#include <sys/socket.h>

#include <netinet/in.h>
#include <string.h>

#include "test_helpers.h"

int main(void)
{
    int status;
    int server_fd = socket(AF_INET, SOCK_DGRAM, 0);
    ERROR_IF(socket, server_fd, == -1);

    struct sockaddr_in addr =
    {
        .sin_family = AF_INET,
        .sin_addr = { .s_addr = htonl(0x7F000001 /* 127.0.0.1 */) },
        .sin_port = htons(0),
    };
    struct sockaddr* saddr = (struct sockaddr*)&addr;
    socklen_t addr_len = sizeof(struct sockaddr_in);
    status = bind(server_fd, saddr, addr_len);
    ERROR_IF(bind, status, == -1);

    status = getsockname(server_fd, saddr, &addr_len);
    ERROR_IF(getsockname, status, == -1);

    int client_fd = socket(AF_INET, SOCK_DGRAM, 0);
    ERROR_IF(socket, client_fd, == -1);
    
    status = connect(client_fd, saddr, addr_len);
    ERROR_IF(connect, status, == -1);

    struct sockaddr_in name;
    struct sockaddr* sname = (struct sockaddr*)&name;
    socklen_t name_len = sizeof(struct sockaddr_in);
    status = getsockname(client_fd, sname, &name_len);
    ERROR_IF(getsockname, status, == -1);

    char *c = "bar";
    status = sendto(server_fd, c, 4, 0, sname, name_len);
    ERROR_IF(sendto, status, == -1);

    struct sockaddr_in from;
    struct sockaddr* sfrom = (struct sockaddr*)&from;
    socklen_t from_len = sizeof(struct sockaddr_in);
    char x[4];
    ssize_t amount = recvfrom(client_fd, x, 4, 0, sfrom, &from_len);
    ERROR_IF(recvfrom, amount, == -1);
    UNEXP_IF(recvfrom, amount, != 4);

    status = strcmp(c, x);
    printf("sendto   %s\n", c);
    printf("recvfrom %s\n", x);
    UNEXP_IF(strcmp, status, != 0);
}
