#include <sys/socket.h>
#include <sys/un.h>
#include <string.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

void print_sockaddr(char *ctx, struct sockaddr *addr, socklen_t len) {
    if (addr->sa_family == AF_UNIX) {
        struct sockaddr_un *un = (struct sockaddr_un *)addr;
        if (len <= sizeof(sa_family_t)) {
            printf("%s [AF_UNIX]: (unnamed)\n", ctx);
        } else {
            printf("%s [AF_UNIX]: %s\n", ctx, un->sun_path);
        }
    } else {
        printf("%s: Unknown address family %d\n", ctx, addr->sa_family);
        exit(1);
    }
}

int main(void)
{
    int status;
    const char* socket_path = "unixpeername.sock";
    unlink(socket_path);

    int listen_fd = socket(AF_UNIX, SOCK_STREAM, 0);
    ERROR_IF(socket, listen_fd, == -1);

    struct sockaddr_un addr = { .sun_family = AF_UNIX };
    struct sockaddr* saddr = (struct sockaddr*)&addr;
    socklen_t addr_len = sizeof(struct sockaddr_un);
    strncpy(addr.sun_path, socket_path, sizeof(addr.sun_path) - 1);

    status = bind(listen_fd, saddr, addr_len);
    ERROR_IF(bind, status, == -1);

    status = listen(listen_fd, 1);
    ERROR_IF(listen, status, == -1);

    struct sockaddr_un check;
    struct sockaddr* scheck = (struct sockaddr*)&check;
    status = getsockname(listen_fd, scheck, &addr_len);
    ERROR_IF(getsockname, status, == -1);
    print_sockaddr("listen sockname", scheck, addr_len);

    int client_fd = socket(AF_UNIX, SOCK_STREAM, 0);
    ERROR_IF(socket, client_fd, == -1);

    status = connect(client_fd, saddr, addr_len);
    ERROR_IF(connect, status, == -1);

    struct sockaddr_un peer;
    struct sockaddr* speer = (struct sockaddr*)&peer;
    socklen_t peer_len = sizeof(struct sockaddr_un);
    status = getpeername(client_fd, speer, &peer_len);
    ERROR_IF(getpeername, status, == -1);
    print_sockaddr("client peername", speer, peer_len);

    status = strcmp(check.sun_path, peer.sun_path);
    UNEXP_IF(strcmp, status, != 0);

    status = getsockname(client_fd, scheck, &addr_len);
    ERROR_IF(getsockname, status, == -1);
    print_sockaddr("client sockname", scheck, addr_len);

    close(client_fd);
    close(listen_fd);
    unlink(socket_path);

    return 0;
}
