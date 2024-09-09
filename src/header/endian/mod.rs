//! endian.h implementation for Redox, following
//! https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/endian.h.html

use crate::platform::types::*;

#[no_mangle]
extern "C" fn be16toh(x: uint16_t) -> uint16_t {
    uint16_t::from_be(x)
}

#[no_mangle]
extern "C" fn be32toh(x: uint32_t) -> uint32_t {
    uint32_t::from_be(x)
}

#[no_mangle]
extern "C" fn be64toh(x: uint64_t) -> uint64_t {
    uint64_t::from_be(x)
}

#[no_mangle]
extern "C" fn htobe16(x: uint16_t) -> uint16_t {
    x.to_be()
}

#[no_mangle]
extern "C" fn htobe32(x: uint32_t) -> uint32_t {
    x.to_be()
}

#[no_mangle]
extern "C" fn htobe64(x: uint64_t) -> uint64_t {
    x.to_be()
}

#[no_mangle]
extern "C" fn htole16(x: uint16_t) -> uint16_t {
    x.to_le()
}

#[no_mangle]
extern "C" fn htole32(x: uint32_t) -> uint32_t {
    x.to_le()
}

#[no_mangle]
extern "C" fn htole64(x: uint64_t) -> uint64_t {
    x.to_le()
}

#[no_mangle]
extern "C" fn le16toh(x: uint16_t) -> uint16_t {
    uint16_t::from_le(x)
}

#[no_mangle]
extern "C" fn le32toh(x: uint32_t) -> uint32_t {
    uint32_t::from_le(x)
}

#[no_mangle]
extern "C" fn le64toh(x: uint64_t) -> uint64_t {
    uint64_t::from_le(x)
}
