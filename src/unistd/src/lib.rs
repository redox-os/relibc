//! unistd implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/unistd.h.html

#![no_std]

extern crate errno;
extern crate platform;
extern crate stdio;
extern crate string;
extern crate sys_time;

use core::{ptr, slice};

use platform::types::*;

pub use brk::*;
pub use getopt::*;
pub use pathconf::*;

mod brk;
mod getopt;
mod pathconf;

pub const R_OK: c_int = 1;
pub const W_OK: c_int = 2;
pub const X_OK: c_int = 4;
pub const F_OK: c_int = 8;

pub const SEEK_SET: c_int = 0;
pub const SEEK_CUR: c_int = 1;
pub const SEEK_END: c_int = 2;

pub const F_ULOCK: c_int = 0;
pub const F_LOCK: c_int = 1;
pub const F_TLOCK: c_int = 2;
pub const F_TEST: c_int = 3;

pub const STDIN_FILENO: c_int = 0;
pub const STDOUT_FILENO: c_int = 1;
pub const STDERR_FILENO: c_int = 2;

#[no_mangle]
pub extern "C" fn _exit(status: c_int) {
    platform::exit(status)
}

// #[no_mangle]
pub extern "C" fn access(path: *const c_char, amode: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn alarm(seconds: c_uint) -> c_uint {
    let mut timer = sys_time::itimerval {
        it_value: sys_time::timeval {
            tv_sec: seconds as time_t,
            tv_usec: 0
        },
        ..Default::default()
    };
    let errno_backup = unsafe { platform::errno };
    let secs = if sys_time::setitimer(sys_time::ITIMER_REAL, &timer, &mut timer) < 0 {
        0
    } else {
        timer.it_value.tv_sec as c_uint + if timer.it_value.tv_usec > 0 { 1 } else { 0 }
    };
    unsafe {
        platform::errno = errno_backup;
    }

    secs
}

#[no_mangle]
pub extern "C" fn chdir(path: *const c_char) -> c_int {
    platform::chdir(path)
}

// #[no_mangle]
pub extern "C" fn chroot(path: *const c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn chown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    platform::chown(path, owner, group)
}

#[no_mangle]
pub extern "C" fn close(fildes: c_int) -> c_int {
    platform::close(fildes)
}

// #[no_mangle]
pub extern "C" fn confstr(name: c_int, buf: *mut c_char, len: size_t) -> size_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn crypt(key: *const c_char, salt: *const c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn dup(fildes: c_int) -> c_int {
    platform::dup(fildes)
}

#[no_mangle]
pub extern "C" fn dup2(fildes: c_int, fildes2: c_int) -> c_int {
    platform::dup2(fildes, fildes2)
}

// #[no_mangle]
pub extern "C" fn encrypt(block: [c_char; 64], edflag: c_int) {
    unimplemented!();
}

// #[no_mangle]
// pub extern "C" fn execl(path: *const c_char, args: *const *mut c_char) -> c_int {
//     unimplemented!();
// }

// #[no_mangle]
// pub extern "C" fn execle(
//   path: *const c_char,
//   args: *const *mut c_char,
//   envp: *const *mut c_char,
// ) -> c_int {
//     unimplemented!();
// }

// #[no_mangle]
// pub extern "C" fn execlp(file: *const c_char, args: *const *mut c_char) -> c_int {
//     unimplemented!();
// }

#[no_mangle]
pub unsafe extern "C" fn execv(path: *const c_char, argv: *const *mut c_char) -> c_int {
    execve(path, argv, platform::environ)
}

#[no_mangle]
pub unsafe extern "C" fn execve(
    path: *const c_char,
    argv: *const *mut c_char,
    envp: *const *mut c_char,
) -> c_int {
    platform::execve(path, argv, envp)
}

// #[no_mangle]
pub extern "C" fn execvp(file: *const c_char, argv: *const *mut c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fchown(fildes: c_int, owner: uid_t, group: gid_t) -> c_int {
    platform::fchown(fildes, owner, group)
}

#[no_mangle]
pub extern "C" fn fchdir(fildes: c_int) -> c_int {
    platform::fchdir(fildes)
}

// #[no_mangle]
pub extern "C" fn fdatasync(fildes: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fork() -> pid_t {
    platform::fork()
}

#[no_mangle]
pub extern "C" fn fsync(fildes: c_int) -> c_int {
    platform::fsync(fildes)
}

#[no_mangle]
pub extern "C" fn ftruncate(fildes: c_int, length: off_t) -> c_int {
    platform::ftruncate(fildes, length)
}

#[no_mangle]
pub extern "C" fn getcwd(buf: *mut c_char, size: size_t) -> *mut c_char {
    platform::getcwd(buf, size)
}

// #[no_mangle]
pub extern "C" fn getdtablesize() -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getegid() -> gid_t {
    platform::getegid()
}

#[no_mangle]
pub extern "C" fn geteuid() -> uid_t {
    platform::geteuid()
}

#[no_mangle]
pub extern "C" fn getgid() -> gid_t {
    platform::getgid()
}

// #[no_mangle]
pub extern "C" fn getgroups(gidsetsize: c_int, grouplist: *mut gid_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn gethostid() -> c_long {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn gethostname(name: *mut c_char, len: size_t) -> c_int {
    platform::gethostname(name, len)
}

// #[no_mangle]
pub extern "C" fn getlogin() -> *mut c_char {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn getlogin_r(name: *mut c_char, namesize: size_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn getpagesize() -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn getpass(prompt: *const c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getpgid(pid: pid_t) -> pid_t {
    platform::getpgid(pid)
}

#[no_mangle]
pub extern "C" fn getpgrp() -> pid_t {
    platform::getpgid(platform::getpid())
}

#[no_mangle]
pub extern "C" fn getpid() -> pid_t {
    platform::getpid()
}

#[no_mangle]
pub extern "C" fn getppid() -> pid_t {
    platform::getppid()
}

// #[no_mangle]
pub extern "C" fn getsid(pid: pid_t) -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getuid() -> uid_t {
    platform::getuid()
}

#[no_mangle]
pub extern "C" fn getwd(path_name: *mut c_char) -> *mut c_char {
    getcwd(path_name, 4096 /* PATH_MAX */)
}

#[no_mangle]
pub extern "C" fn isatty(fd: c_int) -> c_int {
    platform::isatty(fd)
}

// #[no_mangle]
pub extern "C" fn lchown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn link(path1: *const c_char, path2: *const c_char) -> c_int {
    platform::link(path1, path2)
}

// #[no_mangle]
pub extern "C" fn lockf(fildes: c_int, function: c_int, size: off_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn lseek(fildes: c_int, offset: off_t, whence: c_int) -> off_t {
    platform::lseek(fildes, offset, whence)
}

// #[no_mangle]
pub extern "C" fn nice(incr: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pause() -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn pipe(fildes: *mut c_int) -> c_int {
    platform::pipe(slice::from_raw_parts_mut(fildes, 2))
}

// #[no_mangle]
pub extern "C" fn pread(fildes: c_int, buf: *mut c_void, nbyte: size_t, offset: off_t) -> ssize_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_atfork(
    prepare: Option<extern "C" fn()>,
    parent: Option<extern "C" fn()>,
    child: Option<extern "C" fn()>,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pwrite(
    fildes: c_int,
    buf: *const c_void,
    nbyte: size_t,
    offset: off_t,
) -> ssize_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn read(fildes: c_int, buf: *const c_void, nbyte: size_t) -> ssize_t {
    use core::slice;
    let buf = unsafe { slice::from_raw_parts_mut(buf as *mut u8, nbyte as usize) };
    platform::read(fildes, buf)
}

// #[no_mangle]
pub extern "C" fn readlink(path: *const c_char, buf: *mut c_char, bufsize: size_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn rmdir(path: *const c_char) -> c_int {
    platform::rmdir(path)
}

#[no_mangle]
pub extern "C" fn setgid(gid: gid_t) -> c_int {
    platform::setregid(gid, gid)
}

#[no_mangle]
pub extern "C" fn setpgid(pid: pid_t, pgid: pid_t) -> c_int {
    platform::setpgid(pid, pgid)
}

// #[no_mangle]
pub extern "C" fn setpgrp() -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setregid(rgid: gid_t, egid: gid_t) -> c_int {
    platform::setregid(rgid, egid)
}

#[no_mangle]
pub extern "C" fn setreuid(ruid: uid_t, euid: uid_t) -> c_int {
    platform::setreuid(ruid, euid)
}

// #[no_mangle]
pub extern "C" fn setsid() -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setuid(uid: uid_t) -> c_int {
    platform::setreuid(uid, uid)
}

#[no_mangle]
pub extern "C" fn sleep(seconds: c_uint) -> c_uint {
    let rqtp = timespec {
        tv_sec: seconds as i64,
        tv_nsec: 0,
    };
    let rmtp = ptr::null_mut();
    platform::nanosleep(&rqtp, rmtp);
    0
}

// #[no_mangle]
pub extern "C" fn swab(src: *const c_void, dest: *mut c_void, nbytes: ssize_t) {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn symlink(path1: *const c_char, path2: *const c_char) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn sync() {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn sysconf(name: c_int) -> c_long {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn tcgetpgrp() -> pid_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn tcsetpgrp(fildes: c_int, pgid_id: pid_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn truncate(path: *const c_char, length: off_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn ttyname(fildes: c_int) -> *mut c_char {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn ttyname_r(fildes: c_int, name: *mut c_char, namesize: size_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ualarm(value: useconds_t, interval: useconds_t) -> useconds_t {
    let mut timer = sys_time::itimerval {
        it_value: sys_time::timeval {
            tv_sec: 0,
            tv_usec: value as suseconds_t
        },
        it_interval: sys_time::timeval {
            tv_sec: 0,
            tv_usec: interval as suseconds_t
        }
    };
    let errno_backup = unsafe { platform::errno };
    let usecs = if sys_time::setitimer(sys_time::ITIMER_REAL, &timer, &mut timer) < 0 {
        0
    } else {
        timer.it_value.tv_sec as useconds_t * 1_000_000 + timer.it_value.tv_usec as useconds_t
    };
    unsafe {
        platform::errno = errno_backup;
    }

    usecs
}

#[no_mangle]
pub extern "C" fn unlink(path: *const c_char) -> c_int {
    platform::unlink(path)
}

#[no_mangle]
pub extern "C" fn usleep(useconds: useconds_t) -> c_int {
    let rqtp = timespec {
        tv_sec: (useconds / 1_000_000) as i64,
        tv_nsec: ((useconds % 1000) * 1000) as i64,
    };
    let rmtp = ptr::null_mut();
    platform::nanosleep(&rqtp, rmtp)
}

// #[no_mangle]
pub extern "C" fn vfork() -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn write(fildes: c_int, buf: *const c_void, nbyte: size_t) -> ssize_t {
    use core::slice;

    let buf = unsafe { slice::from_raw_parts(buf as *const u8, nbyte as usize) };
    platform::write(fildes, buf)
}

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
