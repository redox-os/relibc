use core::slice;
use syscall;
use c_str;
use types::*;

pub unsafe fn cstr_to_slice<'a>(buf: *const c_char) -> &'a [u8] {
    slice::from_raw_parts(buf as *const u8, ::strlen(buf) as usize)
}

pub fn brk(addr: *const c_void) -> {
    syscall::brk(addr as usize)? as c_int

pub fn chdir(path: *const c_char) -> c_int {
    syscall::chdir(cstr_to_slice(path))? as c_int
}

pub fn chown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    let fd = syscall::open(cstr_to_slice(path));
    syscall::fchown(fd, owner, group)? as c_int

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
    syscall::fchown(owner, group)? as c_int
}

pub fn fchdir(fd: c_int) -> c_int {
    let path = fpath(fd as usize, &[]).unwrap();
    syscall::chdir(path)? as c_int
}

pub fn open(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    let path = unsafe { c_str(path) };
    syscall::open(path, (oflag as usize) | (mode as usize)).unwrap() as c_int
}

pub fn write(fd: c_int, buf: &[u8]) -> ssize_t {
    syscall::write(fd as usize, buf);
    buf.len() as ssize_t
}
