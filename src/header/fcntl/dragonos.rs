use crate::platform::types::*;

pub const O_RDONLY: c_int = 0x0000;
pub const O_WRONLY: c_int = 0x0001;
pub const O_RDWR: c_int = 0x0002;
pub const O_ACCMODE: c_int = 0x0003;
pub const O_CREAT: c_int = 0x0040;
pub const O_EXCL: c_int = 0x0080;
pub const O_TRUNC: c_int = 0x0200;
pub const O_APPEND: c_int = 0x0400;
pub const O_NONBLOCK: c_int = 0x0800;
pub const O_DIRECTORY: c_int = 0x1_0000;
pub const O_NOFOLLOW: c_int = 0x2_0000;
pub const O_CLOEXEC: c_int = 0x8_0000;
pub const O_PATH: c_int = 0x20_0000;

pub const FD_CLOEXEC: c_int = 0x8_0000;
