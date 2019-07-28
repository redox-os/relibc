use crate::platform::types::*;

pub const EPOLL_CLOEXEC: c_int = 0x0100_0000;

pub const EPOLLIN: c_uint = 1;
pub const EPOLLPRI: c_uint = 0;
pub const EPOLLOUT: c_uint = 2;
pub const EPOLLRDNORM: c_uint = 0;
pub const EPOLLNVAL: c_uint = 0;
pub const EPOLLRDBAND: c_uint = 0;
pub const EPOLLWRNORM: c_uint = 0;
pub const EPOLLWRBAND: c_uint = 0;
pub const EPOLLMSG: c_uint = 0;
pub const EPOLLERR: c_uint = 0;
pub const EPOLLHUP: c_uint = 0;
pub const EPOLLRDHUP: c_uint = 0;
pub const EPOLLEXCLUSIVE: c_uint = 0;
pub const EPOLLWAKEUP: c_uint = 0;
pub const EPOLLONESHOT: c_uint = 0;
pub const EPOLLET: c_uint = 0;
