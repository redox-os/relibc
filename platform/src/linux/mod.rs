use types::*;

const AT_FDCWD: c_int = -100;

pub fn brk(addr: *const c_void) -> c_int {
    unsafe {
        syscall!(BRK, addr) as c_int
    }
}

pub fn chdir(path: *const c_char) -> c_int {
    unsafe {
        syscall!(CHDIR, path) as c_int
    }
}

pub fn chown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    unsafe {
        syscall!(CHOWN, owner as u32, group as u32) as c_int
    }
}

pub fn close(fildes: c_int) -> c_int {
    unsafe {
        syscall!(CLOSE, fildes) as c_int
    }
}

pub fn dup(fildes: c_int) -> c_int {
    unsafe {
        syscall!(DUP, fildes) as c_int
    }
}

pub fn dup2(fildes: c_int, fildes2:c_int) -> c_int {
    unsafe {
        syscall!(DUP2, fildes, fildes2) as c_int
    }
}

pub fn exit(status: c_int) -> ! {
    unsafe {
        syscall!(EXIT, status);
    }
    loop {}
}

pub fn fchown(fildes: c_int, owner: uid_t, group: gid_t) -> c_int {
    unsafe {
        syscall!(FCHOWN, owner, group) as c_int
    }
}

pub fn fchdir(fildes: c_int) -> c_int {
    unsafe {
        syscall!(FCHDIR, fildes) as c_int
    }
}

pub fn fsync(fildes: c_int) -> c_int {
    unsafe {
        syscall!(FSYNC, fildes) as c_int
    }
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
