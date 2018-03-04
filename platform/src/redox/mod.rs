use syscall;

use c_str;
use types::*;

pub fn close(fd: c_int) -> c_int {
    syscall::close(fd as usize);
    0
}

pub fn exit(status: c_int) -> ! {
    syscall::exit(status as usize);
    loop {}
}

pub fn open(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    let path = unsafe { c_str(path) };
    syscall::open(path, (oflag as usize) | (mode as usize)).unwrap() as c_int
}

pub fn write(fd: c_int, buf: &[u8]) -> ssize_t {
    syscall::write(fd as usize, buf);
    buf.len() as ssize_t
}
