use crate::platform::types::{uint16_t, uint32_t};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/htonl.html>.
#[unsafe(no_mangle)]
pub extern "C" fn htonl(hostlong: uint32_t) -> uint32_t {
    hostlong.to_be()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/htonl.html>.
#[unsafe(no_mangle)]
pub extern "C" fn htons(hostshort: uint16_t) -> uint16_t {
    hostshort.to_be()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/htonl.html>.
#[unsafe(no_mangle)]
pub extern "C" fn ntohl(netlong: uint32_t) -> uint32_t {
    u32::from_be(netlong)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/htonl.html>.
#[unsafe(no_mangle)]
pub extern "C" fn ntohs(netshort: uint16_t) -> uint16_t {
    u16::from_be(netshort)
}
