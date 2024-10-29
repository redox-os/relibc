//! `endian.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/endian.h.html>.

use crate::platform::types::*;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/be16toh.html>.
#[no_mangle]
pub extern "C" fn be16toh(x: uint16_t) -> uint16_t {
    uint16_t::from_be(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/be16toh.html>.
#[no_mangle]
pub extern "C" fn be32toh(x: uint32_t) -> uint32_t {
    uint32_t::from_be(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/be16toh.html>.
#[no_mangle]
pub extern "C" fn be64toh(x: uint64_t) -> uint64_t {
    uint64_t::from_be(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/be16toh.html>.
#[no_mangle]
pub extern "C" fn htobe16(x: uint16_t) -> uint16_t {
    x.to_be()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/be16toh.html>.
#[no_mangle]
pub extern "C" fn htobe32(x: uint32_t) -> uint32_t {
    x.to_be()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/be16toh.html>.
#[no_mangle]
pub extern "C" fn htobe64(x: uint64_t) -> uint64_t {
    x.to_be()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/be16toh.html>.
#[no_mangle]
pub extern "C" fn htole16(x: uint16_t) -> uint16_t {
    x.to_le()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/be16toh.html>.
#[no_mangle]
pub extern "C" fn htole32(x: uint32_t) -> uint32_t {
    x.to_le()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/be16toh.html>.
#[no_mangle]
pub extern "C" fn htole64(x: uint64_t) -> uint64_t {
    x.to_le()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/be16toh.html>.
#[no_mangle]
pub extern "C" fn le16toh(x: uint16_t) -> uint16_t {
    uint16_t::from_le(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/be16toh.html>.
#[no_mangle]
pub extern "C" fn le32toh(x: uint32_t) -> uint32_t {
    uint32_t::from_le(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/be16toh.html>.
#[no_mangle]
pub extern "C" fn le64toh(x: uint64_t) -> uint64_t {
    uint64_t::from_le(x)
}
