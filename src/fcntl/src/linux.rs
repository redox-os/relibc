use platform::types::*;

pub const O_RDONLY: c_int =     0x0000;
pub const O_WRONLY: c_int =     0x0001;
pub const O_RDWR: c_int =       0x0002;
pub const O_CREAT: c_int =      0x0040;
pub const O_TRUNC: c_int =      0x0200;
pub const O_ACCMODE: c_int = O_RDONLY | O_WRONLY | O_RDWR;
