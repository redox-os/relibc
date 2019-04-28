use syscall::flag::{EVENT_READ, EVENT_WRITE};
use header::fcntl::O_CLOEXEC;
use platform::types::*;

pub const EPOLL_CLOEXEC: c_int = O_CLOEXEC;

pub const EPOLLIN: c_uint = EVENT_READ as c_uint;
pub const EPOLLPRI: c_uint = 0;
pub const EPOLLOUT: c_uint = EVENT_WRITE as c_uint;
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
