//! `netinet/tcp.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_tcp.h.html>.

use crate::platform::types::c_int;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netinet_tcp.h.html>.
///
/// Avoid coalescing of small segments.
pub const TCP_NODELAY: c_int = 1;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/tcp.7.html>.
///
/// The maximum segment size for outgoing TCP packets.
pub const TCP_MAXSEG: c_int = 2;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/tcp.7.html>.
///
/// The time (in seconds) the connection needs to remain idle before TCP starts
/// sending keepalive probes, if the socket option `SO_KEEPALIVE` has been set
/// on this socket.
pub const TCP_KEEPIDLE: c_int = 4;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/tcp.7.html>.
///
/// The time (in seconds) between individual keepalive probes.
pub const TCP_KEEPINTVL: c_int = 5;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/tcp.7.html>.
///
/// The maximum number of keepalive probes TCP should send before dropping the
/// connection.
pub const TCP_KEEPCNT: c_int = 6;
