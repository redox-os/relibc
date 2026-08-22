//! `sys/un.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_un.h.html>.

use crate::{header::bits_safamily_t::sa_family_t, platform::types::c_char};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_un.h.html>.
///
/// # Implementation
/// Size of `sun_path` is `108` bytes.
#[repr(C)]
pub struct sockaddr_un {
    /// Address family.
    pub sun_family: sa_family_t,
    /// Socket pathname storage.
    pub sun_path: [c_char; 108],
}
unsafe impl plain::Plain for sockaddr_un {}
