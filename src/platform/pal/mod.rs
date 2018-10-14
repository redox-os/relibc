use super::types::*;
use c_str::CStr;
use header::dirent::dirent;
//use header::sys_resource::rusage;
use header::sys_select::fd_set;
use header::sys_stat::stat;
use header::sys_time::{itimerval, timeval, timezone};
//use header::sys_times::tms;
use header::sys_utsname::utsname;
use header::termios::termios;
use header::time::timespec;

pub use self::signal::PalSignal;
mod signal;

pub use self::socket::PalSocket;
mod socket;

pub trait Pal {
    fn access(path: &CStr, mode: c_int) -> c_int;

    fn brk(addr: *mut c_void) -> *mut c_void;

    fn chdir(path: &CStr) -> c_int;

    fn chmod(path: &CStr, mode: mode_t) -> c_int;

    fn chown(path: &CStr, owner: uid_t, group: gid_t) -> c_int;

    fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> c_int;

    fn close(fildes: c_int) -> c_int;

    fn dup(fildes: c_int) -> c_int;

    fn dup2(fildes: c_int, fildes2: c_int) -> c_int;

    unsafe fn execve(path: &CStr, argv: *const *mut c_char, envp: *const *mut c_char) -> c_int;

    fn exit(status: c_int) -> !;

    fn fchdir(fildes: c_int) -> c_int;

    fn fchmod(fildes: c_int, mode: mode_t) -> c_int;

    fn fchown(fildes: c_int, owner: uid_t, group: gid_t) -> c_int;

    fn flock(fd: c_int, operation: c_int) -> c_int;

    fn fstat(fildes: c_int, buf: *mut stat) -> c_int;

    fn fcntl(fildes: c_int, cmd: c_int, arg: c_int) -> c_int;

    fn fork() -> pid_t;

    fn fsync(fildes: c_int) -> c_int;

    fn ftruncate(fildes: c_int, length: off_t) -> c_int;

    fn futex(addr: *mut c_int, op: c_int, val: c_int) -> c_int;

    fn futimens(fd: c_int, times: *const timespec) -> c_int;

    fn utimens(path: &CStr, times: *const timespec) -> c_int;

    fn getcwd(buf: *mut c_char, size: size_t) -> *mut c_char;

    fn getdents(fd: c_int, dirents: *mut dirent, bytes: usize) -> c_int;

    fn getegid() -> gid_t;

    fn geteuid() -> uid_t;

    fn getgid() -> gid_t;

    fn gethostname(name: *mut c_char, len: size_t) -> c_int;

    fn getpgid(pid: pid_t) -> pid_t;

    fn getpid() -> pid_t;

    fn getppid() -> pid_t;

    fn gettimeofday(tp: *mut timeval, tzp: *mut timezone) -> c_int;

    fn getuid() -> uid_t;

    fn isatty(fd: c_int) -> c_int;

    fn link(path1: &CStr, path2: &CStr) -> c_int;

    fn lseek(fildes: c_int, offset: off_t, whence: c_int) -> off_t;

    fn mkdir(path: &CStr, mode: mode_t) -> c_int;

    fn mkfifo(path: &CStr, mode: mode_t) -> c_int;

    unsafe fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: c_int,
        flags: c_int,
        fildes: c_int,
        off: off_t,
    ) -> *mut c_void;

    unsafe fn munmap(addr: *mut c_void, len: usize) -> c_int;

    fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> c_int;

    fn open(path: &CStr, oflag: c_int, mode: mode_t) -> c_int;

    fn pipe(fildes: &mut [c_int]) -> c_int;

    fn read(fildes: c_int, buf: &mut [u8]) -> ssize_t;

    //fn readlink(pathname: &CStr, out: &mut [u8]) -> ssize_t;

    fn realpath(pathname: &CStr, out: &mut [u8]) -> c_int;

    fn rename(old: &CStr, new: &CStr) -> c_int;

    fn rmdir(path: &CStr) -> c_int;

    fn select(
        nfds: c_int,
        readfds: *mut fd_set,
        writefds: *mut fd_set,
        exceptfds: *mut fd_set,
        timeout: *mut timeval,
    ) -> c_int;

    fn setpgid(pid: pid_t, pgid: pid_t) -> c_int;

    fn setregid(rgid: gid_t, egid: gid_t) -> c_int;

    fn setreuid(ruid: uid_t, euid: uid_t) -> c_int;

    fn tcgetattr(fd: c_int, out: *mut termios) -> c_int;

    fn tcsetattr(fd: c_int, act: c_int, value: *const termios) -> c_int;

    fn unlink(path: &CStr) -> c_int;

    fn waitpid(pid: pid_t, stat_loc: *mut c_int, options: c_int) -> pid_t;

    fn write(fildes: c_int, buf: &[u8]) -> ssize_t;
}
