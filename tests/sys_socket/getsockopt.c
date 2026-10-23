#include <arpa/inet.h>
#include <stdio.h>

#include "test_helpers.h"

void test_getsockopt(int fd, int option_name) {
    int optval = -1;
    socklen_t optlen = sizeof(optval);

    if (getsockopt(fd, SOL_SOCKET, option_name, &optval, &optlen) < 0) {
        perror("getsockopt");
        int status = close(fd);
        ERROR_IF(close, status, == -1);

        exit(EXIT_FAILURE);
    }

    printf("%d: %d\n", option_name, optval);
}

int main(void)
{
    int status;
    int tcp_fd = socket(AF_INET, SOCK_STREAM, 0);

    test_getsockopt(tcp_fd, SO_DOMAIN);
    test_getsockopt(tcp_fd, SO_TYPE);
    status = close(tcp_fd);
    ERROR_IF(close, status, == -1);

    int udp_fd = socket(AF_INET, SOCK_DGRAM, 0);
    ERROR_IF(socket, udp_fd, == -1);

    test_getsockopt(udp_fd, SO_DOMAIN);
    test_getsockopt(udp_fd, SO_TYPE);
    status = close(udp_fd);
    ERROR_IF(close, status, == -1);

    return EXIT_SUCCESS;
}
