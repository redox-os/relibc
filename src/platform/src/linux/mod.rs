use types::*;

const AT_FDCWD: c_int = -100;

pub fn brk(addr: *const c_void) -> c_int {
    unsafe {
        let newbrk = syscall!(BRK, addr);
        if newbrk < addr as usize {
            -1
        } else {
            0
        }
    }
}

pub fn chdir(path: *const c_char) -> c_int {
    unsafe { syscall!(CHDIR, path) as c_int }
}

pub fn chown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    unsafe { syscall!(FCHOWNAT, AT_FDCWD, path, owner as u32, group as u32) as c_int }
}

pub fn close(fildes: c_int) -> c_int {
    unsafe { syscall!(CLOSE, fildes) as c_int }
}

pub fn dup(fildes: c_int) -> c_int {
    unsafe { syscall!(DUP, fildes) as c_int }
}

pub fn dup2(fildes: c_int, fildes2: c_int) -> c_int {
    unsafe { syscall!(DUP3, fildes, fildes2, 0) as c_int }
}

pub fn exit(status: c_int) -> ! {
    unsafe {
        syscall!(EXIT, status);
    }
    loop {}
}

pub fn fchown(fildes: c_int, owner: uid_t, group: gid_t) -> c_int {
    unsafe { syscall!(FCHOWN, fildes, owner, group) as c_int }
}

pub fn fchdir(fildes: c_int) -> c_int {
    unsafe { syscall!(FCHDIR, fildes) as c_int }
}

pub fn fsync(fildes: c_int) -> c_int {
    unsafe { syscall!(FSYNC, fildes) as c_int }
}

pub fn ftruncate(fildes: c_int, length: off_t) -> c_int {
    unsafe { syscall!(FTRUNCATE, fildes, length) as c_int }
}

pub fn getcwd(buf: *mut c_char, size: size_t) -> *mut c_char {
    unsafe {
        syscall!(GETCWD, buf, size);
        buf as *mut c_char
    }
}

pub fn getegid() -> gid_t {
    unsafe { syscall!(GETEGID) }
}

pub fn geteuid() -> uid_t {
    unsafe { syscall!(GETEUID) }
}

pub fn getgid() -> gid_t {
    unsafe { syscall!(GETGID) }
}

pub fn getpgid(pid: pid_t) -> pid_t {
    unsafe { syscall!(GETPGID, pid) }
}

pub fn getpid() -> pid_t {
    unsafe { syscall!(GETPID) }
}

pub fn getppid() -> pid_t {
    unsafe { syscall!(GETPPID) }
}

pub fn getuid() -> uid_t {
    unsafe { syscall!(GETUID) }
}

pub fn link(path1: *const c_char, path2: *const c_char) -> c_int {
    unsafe { syscall!(LINKAT, AT_FDCWD, path1, path2) as c_int }
}

pub fn open(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    unsafe { syscall!(OPENAT, AT_FDCWD, path, oflag, mode) as c_int }
}

pub fn write(fildes: c_int, buf: &[u8]) -> ssize_t {
    unsafe { syscall!(WRITE, fildes, buf.as_ptr(), buf.len()) as ssize_t }
}
