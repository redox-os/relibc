use core::ptr;
use core::slice;
use syscall;
use syscall::flag::*;

use c_str;
use errno;
use types::*;

pub fn e(sys: Result<usize, syscall::Error>) -> usize {
    match sys {
        Ok(ok) => ok,
        Err(err) => {
            unsafe {
                errno = err.errno as c_int;
            }
            !0
        }
    }
}

pub fn brk(addr: *const c_void) -> c_int {
    e(unsafe { syscall::brk(addr as usize) }) as c_int
}

pub fn chdir(path: *const c_char) -> c_int {
    let path = unsafe { c_str(path) };
    e(syscall::chdir(path)) as c_int
}

pub fn chown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    let path = unsafe { c_str(path) };
    let fd = syscall::open(path, 0x0001).unwrap();
    e(syscall::fchown(fd as usize, owner as u32, group as u32)) as c_int
}

pub fn close(fd: c_int) -> c_int {
    e(syscall::close(fd as usize)) as c_int
}

pub fn dup(fd: c_int) -> c_int {
    e(syscall::dup(fd as usize, &[])) as c_int
}

pub fn dup2(fd1: c_int, fd2: c_int) -> c_int {
    e(syscall::dup2(fd1 as usize, fd2 as usize, &[])) as c_int
}

pub fn exit(status: c_int) -> ! {
    syscall::exit(status as usize);
    loop {}
}

pub fn fchown(fd: c_int, owner: uid_t, group: gid_t) -> c_int {
    e(syscall::fchown(fd as usize, owner as u32, group as u32)) as c_int
}

pub fn fchdir(fd: c_int) -> c_int {
    let path: &mut [u8] = &mut [0; 4096];
    if e(syscall::fpath(fd as usize, path)) == !0 {
        !0
    } else {
        e(syscall::chdir(path)) as c_int
    }
}

pub fn fork() -> pid_t {
    e(unsafe { syscall::clone(0) }) as pid_t
}

pub fn fsync(fd: c_int) -> c_int {
    e(syscall::fsync(fd as usize)) as c_int
}

pub fn ftruncate(fd: c_int, len: off_t) -> c_int {
    e(syscall::ftruncate(fd as usize, len as usize)) as c_int
}

pub fn getcwd(buf: *mut c_char, size: size_t) -> *mut c_char {
    let buf_slice = unsafe { slice::from_raw_parts_mut(buf as *mut u8, size as usize) };
    if e(syscall::getcwd(buf_slice)) == !0 {
        ptr::null_mut()
    } else {
        buf
    }
}

pub fn getegid() -> gid_t {
    e(syscall::getegid()) as gid_t
}

pub fn geteuid() -> uid_t {
    e(syscall::geteuid()) as uid_t
}

pub fn getgid() -> gid_t {
    e(syscall::getgid()) as gid_t
}

pub fn getpgid(pid: pid_t) -> pid_t {
    e(syscall::getpgid(pid as usize)) as pid_t
}

pub fn getpid() -> pid_t {
    e(syscall::getpid()) as pid_t
}

pub fn getppid() -> pid_t {
    e(syscall::getppid()) as pid_t
}

pub fn getuid() -> uid_t {
    e(syscall::getuid()) as pid_t
}

pub fn link(path1: *const c_char, path2: *const c_char) -> c_int {
    let path1 = unsafe { c_str(path1) };
    let path2 = unsafe { c_str(path2) };
    e(unsafe { syscall::link(path1.as_ptr(), path2.as_ptr()) }) as c_int
}

pub fn mkdir(path: *const c_char, mode: mode_t) -> c_int {
    let flags = O_CREAT | O_EXCL | O_CLOEXEC | O_DIRECTORY | mode as usize & 0o777;
    let path = unsafe { c_str(path) };
    match syscall::open(path, flags) {
        Ok(fd) => {
            syscall::close(fd);
            0
        },
        Err(err) => {
            e(Err(err)) as c_int
        }
    }
}


pub fn open(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    let path = unsafe { c_str(path) };
    e(syscall::open(path, (oflag as usize) | (mode as usize))) as c_int
}

pub fn pipe(mut fds: [c_int; 2]) -> c_int {
    let mut usize_fds: [usize; 2] = [0; 2];
    let res = e(syscall::pipe2(&mut usize_fds, 0));
    fds[0] = usize_fds[0] as c_int;
    fds[1] = usize_fds[1] as c_int;
    res as c_int
}

pub fn read(fd: c_int, buf: &mut [u8]) -> ssize_t {
    e(syscall::read(fd as usize, buf)) as ssize_t
}

pub fn rmdir(path: *const c_char) -> c_int {
    let path = unsafe { c_str(path) };
    e(syscall::rmdir(path)) as c_int
}

pub fn write(fd: c_int, buf: &[u8]) -> ssize_t {
    e(syscall::write(fd as usize, buf)) as ssize_t
}
