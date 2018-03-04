use types::*;

const AT_FDCWD: c_int = -100;

pub fn close(fildes: c_int) -> c_int {
    unsafe {
        syscall!(CLOSE, fildes) as c_int
    }
}

pub fn exit(status: c_int) -> ! {
    unsafe {
        syscall!(EXIT, status);
    }
    loop {}
}

#[cfg(target_arch = "x86_64")]
pub fn open(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    unsafe {
        syscall!(OPEN, path, oflag, mode) as c_int
    }
}

#[cfg(target_arch = "aarch64")]
pub fn open(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    unsafe {
        syscall!(OPENAT, AT_FDCWD, path, oflag, mode) as c_int
    }
}


pub fn write(fildes: c_int, buf: &[u8]) -> ssize_t {
    unsafe {
        syscall!(WRITE, fildes, buf.as_ptr(), buf.len()) as ssize_t
    }
}
