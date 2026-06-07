use crate::platform::types::c_int;

// Socket types

/// Byte-stream socket.
pub const SOCK_STREAM: c_int = 1;
/// Datagram socket.
pub const SOCK_DGRAM: c_int = 2;
/// Raw Protocol Interface.
pub const SOCK_RAW: c_int = 3;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/socket.2.html>.
///
/// Provides a reliable datagram layer that does not guarantee ordering.
pub const SOCK_RDM: c_int = 4;
/// Sequenced-packet socket.
pub const SOCK_SEQPACKET: c_int = 5;

// End of socket types

// Socket creation flags (for use in `socket()`, `socketpair()` and `accept4()`)

/// Create a socket file descriptor with the `O_NONBLOCK` flag atomically set
/// on the new open file description.
pub const SOCK_NONBLOCK: c_int = 0o4_000;
/// Create a socket file descriptor wit the `FD_CLOEXEC` flag atomically set
/// on that file descriptor.
pub const SOCK_CLOEXEC: c_int = 0o2_000_000;

// End of socket creation flags

// `level` argument of `getsockopt()` and `setsockopt()`

/// Options to be accessed at socket level, not protocol level.
pub const SOL_SOCKET: c_int = 1;

// End of `level` argument

// `option_name` argument for `getsockopt()` and `setsockopt()`

/// Debugging information is recorded.
pub const SO_DEBUG: c_int = 1;
/// Reuse of local addresses is supported.
pub const SO_REUSEADDR: c_int = 2;
/// Socket type.
pub const SO_TYPE: c_int = 3;
/// Socket error status.
pub const SO_ERROR: c_int = 4;
/// Bypass normal routing.
pub const SO_DONTROUTE: c_int = 5;
/// Transmission of broadcast messages is supported.
pub const SO_BROADCAST: c_int = 6;
/// Send buffer size.
pub const SO_SNDBUF: c_int = 7;
/// Receive buffer size.
pub const SO_RCVBUF: c_int = 8;
/// Connections are kept alive with periodic messages.
pub const SO_KEEPALIVE: c_int = 9;
/// Out-of-band data is transmitted inline.
pub const SO_OOBINLINE: c_int = 10;
/// Non-POSIX, found in LwIP.
///
/// Don't create UDP checksum.
pub const SO_NO_CHECK: c_int = 11;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/socket.7.html>.
///
/// Set the protocol-defined priority for all packets to be sent on this
/// socket.
pub const SO_PRIORITY: c_int = 12;
/// Socket lingers on close.
pub const SO_LINGER: c_int = 13;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/socket.7.html>.
///
/// Enable BSD bug-to-bug compatibility.
pub const SO_BSDCOMPAT: c_int = 14;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/socket.7.html>.
///
/// Permits multiple `AF_INET` or `AF_INET6` sockets to be bound to an
/// identical socket address.
pub const SO_REUSEPORT: c_int = 15;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/socket.7.html>.
///
/// Enable or disable the receiving of the `SCM_CREDENTIALS` control message.
pub const SO_PASSCRED: c_int = 16;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/socket.7.html>.
///
/// Return the credentials of the peer process connected to this socket.
pub const SO_PEERCRED: c_int = 17;
/// Receive "low water mark".
pub const SO_RCVLOWAT: c_int = 18;
/// Send "low water mark".
pub const SO_SNDLOWAT: c_int = 19;
/// Receive timeout.
pub const SO_RCVTIMEO: c_int = 20;
/// Send timeout.
pub const SO_SNDTIMEO: c_int = 21;
/// Socket is accepting connections.
pub const SO_ACCEPTCONN: c_int = 30;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/SO_PEERSEC.2const.html>.
///
/// Get the security context of a peer socket.
pub const SO_PEERSEC: c_int = 31;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/socket.7.html>.
///
/// A privileged (`CAP_NET_ADMIN`) process can perform the same task as
/// `SO_SNDBUF`, but the `wmem_max` limit can be overridden.
pub const SO_SNDBUFFORCE: c_int = 32;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/socket.7.html>.
///
/// A privileged (`CAP_NET_ADMIN`) process can perform the same task as
/// `SO_RCVBUF`, but the `rmem_max` limit can be overridden.
pub const SO_RCVBUFFORCE: c_int = 33;
/// Socket protocol.
pub const SO_PROTOCOL: c_int = 38;
/// Socket domain.
pub const SO_DOMAIN: c_int = 39;

// End of `option_name` argument

/// The maximum backlog queue length.
pub const SOMAXCONN: c_int = 128;

// `msg_flags`

/// Control data truncated.
pub const MSG_CTRUNC: c_int = 8;
/// Send without using routing tables.
pub const MSG_DONTROUTE: c_int = 4;
/// Terminates a record (if supported by the protocol).
pub const MSG_EOR: c_int = 128;
/// Out-of-band data.
pub const MSG_OOB: c_int = 1;
/// Leave received data in queue.
pub const MSG_PEEK: c_int = 2;
/// Normal data truncated.
pub const MSG_TRUNC: c_int = 32;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/send.2.html>.
///
/// Enables nonblocking operation.
pub const MSG_DONTWAIT: c_int = 64;
/// Attempt to fill the read buffer.
pub const MSG_WAITALL: c_int = 256;
/// No SIGPIPE generated when an attempt to send is made on a stream-oriented
/// socket that is no longer connected.
pub const MSG_NOSIGNAL: c_int = 0x4000;
/// Atomically set the `FD_CLOEXEC` flag on any file descriptors created via
/// `SCM_RIGHTS` during `recvmsg()`.
pub const MSG_CMSG_CLOEXEC: c_int = 0x40000000;

// End of `mag_flags`

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/IP_ADD_SOURCE_MEMBERSHIP.2const.html>.
///
/// Join a multicast group and allow receiving data only from a specified
/// source.
pub const IP_ADD_SOURCE_MEMBERSHIP: c_int = 70;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/IP_DROP_SOURCE_MEMBERSHIP.2const.html>.
///
/// Leave a source-specific multicast group.
pub const IP_DROP_SOURCE_MEMBERSHIP: c_int = 71;
/// Non-POSIX, see <https://datatracker.ietf.org/doc/html/rfc3678#section-5.1.2>.
///
/// Join a source-specific group.
pub const MCAST_JOIN_SOURCE_GROUP: c_int = 46;
/// Non-POSIX, see <https://datatracker.ietf.org/doc/html/rfc3678#section-5.1.2>.
///
/// Leave a source-specific group.
pub const MCAST_LEAVE_SOURCE_GROUP: c_int = 47;

// Address family constants (AF)

/// Internet domain sockets for use with IPv4 sockets.
pub const AF_INET: c_int = 2;
/// Internet domain sockets for use with IPv6 sockets.
pub const AF_INET6: c_int = 10;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/unix.7.html>.
///
/// Alias for `AF_UNIX`.
pub const AF_LOCAL: c_int = AF_UNIX;
/// UNIX domain sockets.
pub const AF_UNIX: c_int = 1;
/// Unspecified.
pub const AF_UNSPEC: c_int = 0;

// End of AF constants

// Protocol family constants (PF)
// Constants historically used by BSDs, identical to AF constants.
// Recommended to use AF constants everywhere instead.
// See <https://www.man7.org/linux/man-pages/man2/socket.2.html>.

/// See `AF_INET`.
pub const PF_INET: c_int = 2;
/// See `AF_INET6`.
pub const PF_INET6: c_int = 10;
/// See `AF_LOCAL`.
pub const PF_LOCAL: c_int = PF_UNIX;
/// See `AF_UNIX`.
pub const PF_UNIX: c_int = 1;
/// See `AF_UNSPEC`.
pub const PF_UNSPEC: c_int = 0;

// End of PF constants

/// Disables further receive operations.
pub const SHUT_RD: c_int = 0;
/// Disables further send and receive operations.
pub const SHUT_RDWR: c_int = 2;
/// Disables further send operations.
pub const SHUT_WR: c_int = 1;

// `cmsg_type` value when `cmsg_level` is `SOL_SOCKET`

/// Indicates that the data array contains the access rights to be sent or
/// received.
pub const SCM_RIGHTS: c_int = 1;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/unix.7.html>.
///
/// Send or receive UNIX credentials.
pub const SCM_CREDENTIALS: c_int = 2;

// End of `cmsg_type` value
