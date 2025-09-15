#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netdb.h>
#include <arpa/inet.h>

#include "test_helpers.h"

int test_getaddrinfo(int ai_flags, size_t* ipv4_addr) {
    struct addrinfo hints, *res;
    int status;

    memset(&hints, 0, sizeof hints);
    hints.ai_family = AF_INET;
    hints.ai_socktype = SOCK_STREAM;
    hints.ai_flags = ai_flags;

    if ((status = getaddrinfo(NULL, "8080", &hints, &res)) != 0) {
        return status;
    }

    *ipv4_addr = ((struct sockaddr_in *)res->ai_addr)->sin_addr.s_addr;
    freeaddrinfo(res);
    return 0;
}

int main() {
    size_t ipv4_addr;
    char addrstr[INET_ADDRSTRLEN];

    int status = test_getaddrinfo(0, &ipv4_addr);
    ERROR_IF(getaddrinfo, status, != 0);
    ERROR_IF(getaddrinfo_ai_addr, ipv4_addr, == 0);
    inet_ntop(AF_INET, &ipv4_addr, addrstr, INET_ADDRSTRLEN);
    printf("local IPv4 address 1: %s\n", addrstr);

    status = test_getaddrinfo(AI_PASSIVE, &ipv4_addr);
    ERROR_IF(getaddrinfo, status, != 0);
    ERROR_IF(getaddrinfo_ai_addr, ipv4_addr, != 0);
    inet_ntop(AF_INET, &ipv4_addr, addrstr, INET_ADDRSTRLEN);
    printf("local IPv4 address 2: %s\n", addrstr);
}
