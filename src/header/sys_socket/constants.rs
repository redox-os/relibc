use crate::platform::types::*;

pub const SOCK_STREAM: c_int = 1;
pub const SOCK_DGRAM: c_int = 2;
pub const SOCK_NONBLOCK: c_int = 0o4_000;
pub const SOCK_CLOEXEC: c_int = 0o2_000_000;

// Other constants
pub const SOCK_SEQPACKET: c_int = 5;

pub const SOL_SOCKET: c_int = 1;

pub const SO_DEBUG: c_int = 1;
pub const SO_REUSEADDR: c_int = 2;
pub const SO_TYPE: c_int = 3;
pub const SO_ERROR: c_int = 4;
pub const SO_DONTROUTE: c_int = 5;
pub const SO_BROADCAST: c_int = 6;
pub const SO_SNDBUF: c_int = 7;
pub const SO_RCVBUF: c_int = 8;
pub const SO_KEEPALIVE: c_int = 9;
pub const SO_OOBINLINE: c_int = 10;
pub const SO_NO_CHECK: c_int = 11;
pub const SO_PRIORITY: c_int = 12;
pub const SO_LINGER: c_int = 13;
pub const SO_BSDCOMPAT: c_int = 14;
pub const SO_REUSEPORT: c_int = 15;
pub const SO_PASSCRED: c_int = 16;
pub const SO_PEERCRED: c_int = 17;
pub const SO_RCVLOWAT: c_int = 18;
pub const SO_SNDLOWAT: c_int = 19;
pub const SO_RCVTIMEO: c_int = 20;
pub const SO_SNDTIMEO: c_int = 21;
pub const SO_ACCEPTCONN: c_int = 30;
pub const SO_PEERSEC: c_int = 31;
pub const SO_SNDBUFFORCE: c_int = 32;
pub const SO_RCVBUFFORCE: c_int = 33;
pub const SO_PROTOCOL: c_int = 38;
pub const SO_DOMAIN: c_int = 39;

pub const SOMAXCONN: c_int = 128;

pub const MSG_CTRUNC: c_int = 8;
pub const MSG_DONTROUTE: c_int = 4;
pub const MSG_EOR: c_int = 128;
pub const MSG_OOB: c_int = 1;
pub const MSG_PEEK: c_int = 2;
pub const MSG_TRUNC: c_int = 32;
pub const MSG_WAITALL: c_int = 256;

pub const AF_INET: c_int = 2;
pub const AF_INET6: c_int = 10;
pub const AF_UNIX: c_int = 1;
pub const AF_UNSPEC: c_int = 0;

pub const PF_INET: c_int = 2;
pub const PF_INET6: c_int = 10;
pub const PF_UNIX: c_int = 1;
pub const PF_UNSPEC: c_int = 0;

pub const SHUT_RD: c_int = 0;
pub const SHUT_RDWR: c_int = 2;
pub const SHUT_WR: c_int = 1;
