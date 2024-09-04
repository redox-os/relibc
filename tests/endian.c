#include <assert.h>
#include <endian.h>
#include <stdint.h>

#include "test_helpers.h"

int main() {
    uint16_t zero_u16 = 0;
    assert(be16toh(zero_u16) == zero_u16);
    assert(htobe16(zero_u16) == zero_u16);
    assert(htole16(zero_u16) == zero_u16);
    assert(le16toh(zero_u16) == zero_u16);

    uint16_t value_be_u16 = 0x0123;
    uint16_t value_le_u16 = 0x2301;
    assert(be16toh(value_be_u16) == value_le_u16);
    assert(htobe16(value_le_u16) == value_be_u16);
    assert(htole16(value_le_u16) == value_le_u16);
    assert(le16toh(value_le_u16) == value_le_u16);

    uint32_t zero_u32 = 0;
    assert(be32toh(zero_u32) == zero_u32);
    assert(htobe32(zero_u32) == zero_u32);
    assert(htole32(zero_u32) == zero_u32);
    assert(le32toh(zero_u32) == zero_u32);

    uint32_t value_be_u32 = 0x01234567;
    uint32_t value_le_u32 = 0x67452301;
    assert(be32toh(value_be_u32) == value_le_u32);
    assert(htobe32(value_le_u32) == value_be_u32);
    assert(htole32(value_le_u32) == value_le_u32);
    assert(le32toh(value_le_u32) == value_le_u32);

    uint64_t zero_u64 = 0;
    assert(be64toh(zero_u64) == zero_u64);
    assert(htobe64(zero_u64) == zero_u64);
    assert(htole64(zero_u64) == zero_u64);
    assert(le64toh(zero_u64) == zero_u64);

    uint64_t value_be_u64 = 0x0123456789ABCDEF;
    uint64_t value_le_u64 = 0xEFCDAB8967452301;
    assert(be64toh(value_be_u64) == value_le_u64);
    assert(htobe64(value_le_u64) == value_be_u64);
    assert(htole64(value_le_u64) == value_le_u64);
    assert(le64toh(value_le_u64) == value_le_u64);
}
