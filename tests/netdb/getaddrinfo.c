// Adapted from https://gist.github.com/jirihnidek/bf7a2363e480491da72301b228b35d5d

#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <netdb.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <arpa/inet.h>

#include "test_helpers.h"

int main(void) {
    struct addrinfo hints, *res;
    int errcode;
    char addrstr[INET6_ADDRSTRLEN];
    void *ptr;

    memset(&hints, 0, sizeof(hints));
    hints.ai_family = PF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;
    hints.ai_flags |= AI_CANONNAME;

    errcode = getaddrinfo("www.redox-os.org", NULL, &hints, &res);
    if (errcode != 0) {
        perror("getaddrinfo");
        exit(EXIT_FAILURE);
    }

    while (res) {
        switch (res->ai_family) {
        case AF_INET:
            ptr = &((struct sockaddr_in *) res->ai_addr)->sin_addr;
            break;
        case AF_INET6:
            ptr = &((struct sockaddr_in6 *) res->ai_addr)->sin6_addr;
            break;
        }
        inet_ntop(res->ai_family, ptr, addrstr, INET6_ADDRSTRLEN);

        printf(
            "IPv%d address: %s (%s)\n",
            res->ai_family == AF_INET6 ? 6 : 4,
            addrstr,
            res->ai_canonname
        );

        res = res->ai_next;
    }
}
