#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <fcntl.h>
#include <errno.h>

#include "test_helpers.h"

// similar to "unixrecv", but this test if write() works before accept().
// "unixrecvfrom" is the similar test to this one, but using SOCK_DGRAM.
// "unixrecvearly" is also the similar test to this one, but using recv().
int main() {
    int status;
    const char* socket_path = "unix_write.sock";
    unlink(socket_path);

    int server_fd = socket(AF_UNIX, SOCK_STREAM, 0);
    ERROR_IF(socket, server_fd, == -1);

    struct sockaddr_un addr = { .sun_family = AF_UNIX };
    strncpy(addr.sun_path, socket_path, sizeof(addr.sun_path) - 1);

    unlink(socket_path);

    status = bind(server_fd, (struct sockaddr*)&addr, sizeof(addr));
    ERROR_IF(bind, status, == -1);

    status = listen(server_fd, 1);
    ERROR_IF(listen, status, == -1);

    int client_fd = socket(AF_UNIX, SOCK_STREAM, 0);
    ERROR_IF(socket, client_fd, == -1);

    status = connect(client_fd, (struct sockaddr*)&addr, sizeof(addr));
    ERROR_IF(connect, status, == -1);

    char *msg = "yes";
    ssize_t ret = write(client_fd, msg, 4);
    ERROR_IF(write, ret, == -1);
    UNEXP_IF(raise, ret, != 4);

    int accepted_fd = accept(server_fd, NULL, NULL);
    ERROR_IF(accept, accepted_fd, == -1);

    char x[4];
    ssize_t amount = read(accepted_fd, x, 4);
    ERROR_IF(read, amount, == -1);

    close(accepted_fd);
    close(client_fd);
    close(server_fd);
    return 0;
}
