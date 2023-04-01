use crate::platform::types::*;

pub const EPOLL_CLOEXEC: c_int = 0x8_0000;

pub const EPOLLIN: c_uint = 0x001;
pub const EPOLLPRI: c_uint = 0x002;
pub const EPOLLOUT: c_uint = 0x004;
pub const EPOLLRDNORM: c_uint = 0x040;
pub const EPOLLNVAL: c_uint = 0x020;
pub const EPOLLRDBAND: c_uint = 0x080;
pub const EPOLLWRNORM: c_uint = 0x100;
pub const EPOLLWRBAND: c_uint = 0x200;
pub const EPOLLMSG: c_uint = 0x400;
pub const EPOLLERR: c_uint = 0x008;
pub const EPOLLHUP: c_uint = 0x010;
pub const EPOLLRDHUP: c_uint = 0x2000;
pub const EPOLLEXCLUSIVE: c_uint = 1 << 28;
pub const EPOLLWAKEUP: c_uint = 1 << 29;
pub const EPOLLONESHOT: c_uint = 1 << 30;
pub const EPOLLET: c_uint = 1 << 31;
