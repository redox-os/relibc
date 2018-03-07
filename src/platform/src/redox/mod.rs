use core::slice;
use syscall;
use c_str;
use types::*;

pub fn brk(addr: *const c_void) -> c_int {
    unsafe {
        syscall::brk(addr as usize).unwrap_or(0-1) as c_int
    }
}

pub fn chdir(path: *const c_char) -> c_int {
    let path = unsafe { c_str(path) };
    syscall::chdir(path).unwrap_or(-1) as c_int
}

pub fn chown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    let path = unsafe { c_str(path) };
    let fd = syscall::open(path, 0x0001).unwrap();
    syscall::fchown(fd as usize, owner as u32, group as u32).unwrap_or(0-1) as c_int
}

pub fn close(fd: c_int) -> c_int {
    syscall::close(fd as usize);
    0
}

pub fn dup(fd: c_int) -> c_int {
    syscall::dup(fd as usize, &[]).unwrap_or(0-1) as c_int
}

pub fn dup2(fd1: c_int, fd2: c_int) -> c_int {
    syscall::dup2(fd1 as usize, fd2 as usize, &[]).unwrap_or(0-1) as c_int
}

pub fn exit(status: c_int) -> ! {
    syscall::exit(status as usize);
    loop {}
}

pub fn fchown(fd: c_int, owner: uid_t, group: gid_t) -> c_int {
    syscall::fchown(fd as usize, owner as u32, group as u32).unwrap_or(0-1) as c_int
}

pub fn fchdir(fd: c_int) -> c_int {
    let path: &mut [u8] = &mut[0; 4096];
    let result = syscall::fpath(fd as usize, path);
    if result.is_ok() {
        syscall::chdir(path).unwrap_or(0-1) as c_int
    } else {
        -1
    }
}

pub fn fsync(fd: c_int) -> c_int {
    syscall::fsync(fd as usize).unwrap_or(0-1) as c_int
}

pub fn ftruncate(fd: c_int, len: off_t) -> c_int {
    syscall::ftruncate(fd as usize, len as usize).unwrap_or(0-1) as c_int
}

pub fn getcwd(buf: *mut c_char, size: size_t) -> *mut c_char {
    // XXX: do something with size maybe
    let rbuf: &mut [u8] = &mut[0; 4096];
    syscall::getcwd(rbuf);
    unsafe {
        let buf = *rbuf.as_ptr() as *mut c_char;
    }
    buf 
}

pub fn getegid() -> gid_t {
    syscall::getegid().unwrap() as gid_t
}

pub fn geteuid() -> uid_t {
    syscall::geteuid().unwrap() as uid_t
}

pub fn getgid() -> gid_t {
    syscall::getgid().unwrap() as gid_t
}

pub fn getpgid(pid: pid_t) -> pid_t {
    syscall::getpgid(pid as usize).unwrap() as pid_t
}

pub fn getpid() -> pid_t {
    syscall::getpid().unwrap() as pid_t
}

pub fn getppid() -> pid_t {
    syscall::getppid().unwrap() as pid_t
}

pub fn getuid() -> uid_t {
    syscall::getuid().unwrap() as pid_t
}

pub fn link(path1: *const c_char, path2: *const c_char) -> c_int {
    let path1 = unsafe { c_str(path1) };
    let path2 = unsafe { c_str(path2) };
    unsafe {
        syscall::link(path1.as_ptr(), path2.as_ptr()).unwrap_or(0-1) as c_int
    }
}

pub fn open(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    let path = unsafe { c_str(path) };
    syscall::open(path, (oflag as usize) | (mode as usize)).unwrap() as c_int
}

pub fn write(fd: c_int, buf: &[u8]) -> ssize_t {
    syscall::write(fd as usize, buf);
    buf.len() as ssize_t
}
