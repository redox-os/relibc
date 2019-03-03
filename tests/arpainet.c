#include <arpa/inet.h>
#include <assert.h>
#include <string.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    uint32_t hl = 0xBADFACED;
    uint32_t nl = htonl(hl);
    assert(nl == 0xEDACDFBA);
    hl = ntohl(nl);
    assert(hl == 0xBADFACED);

    uint16_t hs = 0xDEAD;
    uint16_t ns = htons(hs);
    assert(ns == 0xADDE);
    hs = ntohs(ns);
    assert(hs == 0xDEAD);

    const char* addr_str = "8.8.4.4";
    struct in_addr* addr = malloc(sizeof addr);
    inet_aton(addr_str, addr);
    assert(strcmp(inet_ntoa(*addr), addr_str) == 0);
}
