//! `netinet/tcp.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_tcp.h.html>.

use crate::platform::types::c_int;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_tcp.h.html>.
pub const TCP_NODELAY: c_int = 1;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/tcp.7.html>.
pub const TCP_MAXSEG: c_int = 2;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/tcp.7.html>.
pub const TCP_KEEPIDLE: c_int = 4;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/tcp.7.html>.
pub const TCP_KEEPINTVL: c_int = 5;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/tcp.7.html>.
pub const TCP_KEEPCNT: c_int = 6;
