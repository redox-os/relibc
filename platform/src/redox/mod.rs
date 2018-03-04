use core::slice;
use syscall;
use c_str;
use types::*;

pub fn brk(addr: *const c_void) -> {
    syscall::brk(addr as usize)? as c_int

 pub fn chdir(path: *const c_char) -> c_int {
    let path = unsafe { c_str(path) };
    syscall::chdir(path)? as c_int
 } 
 

pub fn chown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    let fd = syscall::open(cstr_to_slice(path));
    syscall::fchown(fd as usize, owner as usize, group as usize)? as c_int

pub fn close(fd: c_int) -> c_int {
    syscall::close(fd as usize);
    0
}

pub fn dup(fd: c_int) -> c_int {
    syscall::dup(fd as usize, &[])? as c_int
}

pub fn dup2(fd1: c_int, fd2) -> c_int {
    syscall::dup2(fd1 as usize, fd2 as usize, &[])? as c_int
}

pub fn exit(status: c_int) -> ! {
    syscall::exit(status as usize);
    loop {}
}

pub fn fchown(fd: c_int, owner: uid_t, group: gid_t) -> c_int {
    syscall::fchown(owner as usize, group as usize)? as c_int
}

pub fn fchdir(fd: c_int) -> c_int {
    let result = fpath(fd as usize, &[]);
    if result.is_ok() {
        syscall::chdir(path)? as c_int
    } else {
        -1
    }
}

pub fn fsync(fd: c_int) -> c_int {
    syscall::fsync(fd as usize)? as c_int
}

pub fn ftruncate(fd: c_int, len: off_t) -> {
    syscall::ftruncate(fd as usize, len as usize)? as c_int
}

pub fn getcwd(buf: *mut c_char, size: size_t) -> {
    // XXX: do something with size maybe
    let rbuf = unsafe { c_str(buf) };
    syscall::getcwd(rbuf);
    unsafe {
        &*(rbuf as *mut [c_char])
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
