#include <assert.h>
#include <endian.h>
#include <stdint.h>

#include "test_helpers.h"

void to_be(uintmax_t in, uint8_t *out, size_t size) {
    for (size_t i = 0; i < size; i++) {
        out[i] = (in >> 8*(size - 1 - i)) & 0xff;
    }
}

void to_le(uintmax_t in, uint8_t *out, size_t size) {
    for (size_t i = 0; i < size; i++) {
        out[i] = (in >> 8*i) & 0xff;
    }
}

int main() {
    uint16_t zero_u16 = 0;
    assert(be16toh(zero_u16) == zero_u16);
    assert(htobe16(zero_u16) == zero_u16);
    assert(htole16(zero_u16) == zero_u16);
    assert(le16toh(zero_u16) == zero_u16);

    uint16_t u16_ne = 0x0123;
    uint16_t u16_be, u16_le;
    to_be(u16_ne, (uint8_t *)&u16_be, sizeof(uint16_t));
    to_le(u16_ne, (uint8_t *)&u16_le, sizeof(uint16_t));
    assert(be16toh(u16_be) == u16_ne);
    assert(htobe16(u16_ne) == u16_be);
    assert(htole16(u16_ne) == u16_le);
    assert(le16toh(u16_le) == u16_ne);

    uint32_t zero_u32 = 0;
    assert(be32toh(zero_u32) == zero_u32);
    assert(htobe32(zero_u32) == zero_u32);
    assert(htole32(zero_u32) == zero_u32);
    assert(le32toh(zero_u32) == zero_u32);

    uint32_t u32_ne = 0x01234567;
    uint32_t u32_be, u32_le;
    to_be(u32_ne, (uint8_t *)&u32_be, sizeof(uint32_t));
    to_le(u32_ne, (uint8_t *)&u32_le, sizeof(uint32_t));
    assert(be32toh(u32_be) == u32_ne);
    assert(htobe32(u32_ne) == u32_be);
    assert(htole32(u32_ne) == u32_le);
    assert(le32toh(u32_le) == u32_ne);

    uint64_t zero_u64 = 0;
    assert(be64toh(zero_u64) == zero_u64);
    assert(htobe64(zero_u64) == zero_u64);
    assert(htole64(zero_u64) == zero_u64);
    assert(le64toh(zero_u64) == zero_u64);

    uint64_t u64_ne = 0x0123456789ABCDEF;
    uint64_t u64_be, u64_le;
    to_be(u64_ne, (uint8_t *)&u64_be, sizeof(uint64_t));
    to_le(u64_ne, (uint8_t *)&u64_le, sizeof(uint64_t));
    assert(be64toh(u64_be) == u64_ne);
    assert(htobe64(u64_ne) == u64_be);
    assert(htole64(u64_ne) == u64_le);
    assert(le64toh(u64_le) == u64_ne);

    /* Test that the BYTE_ORDER, LITTLE_ENDIAN and BIG_ENDIAN macros are available */
    /* It is in principle possible to have further endiannesses, like PDP_ENDIAN */
    if (u64_ne == u64_le) {
        assert(BYTE_ORDER == LITTLE_ENDIAN);
        assert(BYTE_ORDER != BIG_ENDIAN);
    }
    if (u64_ne == u64_be) {
        assert(BYTE_ORDER == BIG_ENDIAN);
        assert(BYTE_ORDER != LITTLE_ENDIAN);
    }
}
