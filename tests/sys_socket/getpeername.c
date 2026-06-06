#include <arpa/inet.h>

#include "test_helpers.h"

void print_sockaddr(char *ctx, struct sockaddr *addr) {
    char ip_string[INET6_ADDRSTRLEN];
    void *raw_ip_addr;
    char *fam;
    in_port_t port;

    if (addr->sa_family == AF_INET) {
        struct sockaddr_in *ipv4 = (struct sockaddr_in *)addr;
        raw_ip_addr = &(ipv4->sin_addr);
        port = ntohs(ipv4->sin_port);
        fam = "AF_INET";
    } else if (addr->sa_family == AF_INET6) {
        struct sockaddr_in6 *ipv6 = (struct sockaddr_in6 *)addr;
        raw_ip_addr = &(ipv6->sin6_addr);
        port = ntohs(ipv6->sin6_port);
        fam = "AF_INET6";
    } else {
        printf("Unknown address family: %d\n", addr->sa_family);
        exit(1);
    }

    if (inet_ntop(addr->sa_family, raw_ip_addr, ip_string, sizeof(ip_string)) == NULL) {
        perror("inet_ntop failed");
        exit(1);
    }

    printf("%s [%s]: %s:%u\n", ctx, fam, ip_string, port);
}

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
    ERROR_IF(bind, status, == -1);

    status = getsockname(listen_fd, saddr, &addr_len);
    ERROR_IF(getsockname, status, == -1);
    print_sockaddr("listen sockname", saddr);

    int client_fd = socket(AF_INET, SOCK_STREAM, 0);
    ERROR_IF(socket, client_fd, == -1);

    status = connect(client_fd, saddr, addr_len);
    ERROR_IF(connect, status, == -1);

    struct sockaddr_in peer;
    struct sockaddr* speer = (struct sockaddr*)&peer;
	socklen_t peer_len = sizeof(struct sockaddr_in);
    status = getpeername(client_fd, speer, &peer_len);
    ERROR_IF(getpeername, status, == -1);
    print_sockaddr("client peername", speer);

    status = memcmp(saddr, speer, sizeof(struct sockaddr_in));
    UNEXP_IF(memcmp, status, != 0);

    status = getsockname(client_fd, saddr, &addr_len);
    ERROR_IF(getsockname, status, == -1);
    print_sockaddr("client sockname", saddr);
}
