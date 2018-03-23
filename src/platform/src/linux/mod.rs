use core::ptr;

use errno;
use types::*;

const AT_FDCWD: c_int = -100;
const AT_REMOVEDIR: c_int = 0x200;

pub fn e(sys: usize) -> usize {
    if (sys as isize) < 0 && (sys as isize) >= -256 {
        unsafe {
            errno = -(sys as isize) as c_int;
        }
        !0
    } else {
        sys
    }
}

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
    e(unsafe { syscall!(CHDIR, path) }) as c_int
}

pub fn chown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    e(unsafe { syscall!(FCHOWNAT, AT_FDCWD, path, owner as u32, group as u32) }) as c_int
}

pub fn close(fildes: c_int) -> c_int {
    e(unsafe { syscall!(CLOSE, fildes) }) as c_int
}

pub fn dup(fildes: c_int) -> c_int {
    e(unsafe { syscall!(DUP, fildes) }) as c_int
}

pub fn dup2(fildes: c_int, fildes2: c_int) -> c_int {
    e(unsafe { syscall!(DUP3, fildes, fildes2, 0) }) as c_int
}

pub fn exit(status: c_int) -> ! {
    unsafe {
        syscall!(EXIT, status);
    }
    loop {}
}

pub fn fchown(fildes: c_int, owner: uid_t, group: gid_t) -> c_int {
    e(unsafe { syscall!(FCHOWN, fildes, owner, group) }) as c_int
}

pub fn fchdir(fildes: c_int) -> c_int {
    e(unsafe { syscall!(FCHDIR, fildes) }) as c_int
}

pub fn fcntl(fildes: c_int, cmd: c_int, arg: c_int) -> c_int {
    e(unsafe { syscall!(FCNTL, fildes, cmd, arg) }) as c_int
}

pub fn fork() -> pid_t {
    e(unsafe { syscall!(CLONE, 17, 0) }) as pid_t
}

pub fn fsync(fildes: c_int) -> c_int {
    e(unsafe { syscall!(FSYNC, fildes) }) as c_int
}

pub fn ftruncate(fildes: c_int, length: off_t) -> c_int {
    e(unsafe { syscall!(FTRUNCATE, fildes, length) }) as c_int
}

pub fn getcwd(buf: *mut c_char, size: size_t) -> *mut c_char {
    if e(unsafe { syscall!(GETCWD, buf, size) }) == !0 {
        ptr::null_mut()
    } else {
        buf
    }
}

pub fn getegid() -> gid_t {
    e(unsafe { syscall!(GETEGID) })
}

pub fn geteuid() -> uid_t {
    e(unsafe { syscall!(GETEUID) })
}

pub fn getgid() -> gid_t {
    e(unsafe { syscall!(GETGID) })
}

pub fn getpgid(pid: pid_t) -> pid_t {
    e(unsafe { syscall!(GETPGID, pid) })
}

pub fn getpid() -> pid_t {
    e(unsafe { syscall!(GETPID) })
}

pub fn getppid() -> pid_t {
    e(unsafe { syscall!(GETPPID) })
}

pub fn getuid() -> uid_t {
    e(unsafe { syscall!(GETUID) })
}

pub fn link(path1: *const c_char, path2: *const c_char) -> c_int {
    e(unsafe { syscall!(LINKAT, AT_FDCWD, path1, AT_FDCWD, path2, 0) }) as c_int
}

pub fn lseek(fildes: c_int, offset: off_t, whence: c_int) -> off_t {
    e(unsafe { syscall!(LSEEK, fildes, offset, whence) }) as off_t
}

pub fn mkdir(path: *const c_char, mode: mode_t) -> c_int {
    e(unsafe { syscall!(MKDIRAT, AT_FDCWD, path, mode) }) as c_int
}

pub fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> c_int {
    e(unsafe { syscall!(NANOSLEEP, rqtp, rmtp) }) as c_int
}

pub fn open(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    e(unsafe { syscall!(OPENAT, AT_FDCWD, path, oflag, mode) }) as c_int
}

pub fn pipe(mut fildes: [c_int; 2]) -> c_int {
    e(unsafe { syscall!(PIPE2, fildes.as_mut_ptr(), 0) }) as c_int
}

pub fn read(fildes: c_int, buf: &mut [u8]) -> ssize_t {
    e(unsafe { syscall!(READ, fildes, buf.as_mut_ptr(), buf.len()) }) as ssize_t
}

pub fn rmdir(path: *const c_char) -> c_int {
    e(unsafe { syscall!(UNLINKAT, AT_FDCWD, path, AT_REMOVEDIR) }) as c_int
}

pub fn setpgid(pid: pid_t, pgid: pid_t) -> c_int {
    e(unsafe { syscall!(SETPGID, pid, pgid) }) as c_int
}

pub fn setregid(rgid: gid_t, egid: gid_t) -> c_int {
    e(unsafe { syscall!(SETREGID, rgid, egid) }) as c_int
}

pub fn setreuid(ruid: uid_t, euid: uid_t) -> c_int {
    e(unsafe { syscall!(SETREUID, ruid, euid) }) as c_int
}

pub fn unlink(path: *const c_char) -> c_int {
    e(unsafe { syscall!(UNLINKAT, AT_FDCWD, path, 0) }) as c_int
}

pub fn waitpid(pid: pid_t, stat_loc: *mut c_int, options: c_int) -> pid_t {
    e(unsafe { syscall!(WAIT4, pid, stat_loc, options, 0) }) as pid_t
}

pub fn write(fildes: c_int, buf: &[u8]) -> ssize_t {
    e(unsafe { syscall!(WRITE, fildes, buf.as_ptr(), buf.len()) }) as ssize_t
}

pub fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> c_int {
    e(unsafe { syscall!(CLOCK_GETTIME, clk_id, tp) }) as c_int
}
