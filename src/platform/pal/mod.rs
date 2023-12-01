use super::types::*;
use crate::{
    c_str::CStr,
    errno::Errno,
    header::{
        dirent::dirent,
        sys_resource::rlimit,
        sys_stat::stat,
        sys_statvfs::statvfs,
        sys_time::{timeval, timezone},
        sys_utsname::utsname,
        time::timespec,
    },
};

pub use self::epoll::PalEpoll;
mod epoll;

pub use self::ptrace::PalPtrace;
mod ptrace;

pub use self::signal::PalSignal;
mod signal;

pub use self::socket::PalSocket;
mod socket;

pub trait Pal {
    fn access(path: CStr, mode: c_int) -> Result<(), Errno>;

    fn brk(addr: *mut c_void) -> Result<*mut c_void, Errno>;

    fn chdir(path: CStr) -> Result<(), Errno>;

    fn chmod(path: CStr, mode: mode_t) -> Result<(), Errno>;

    fn chown(path: CStr, owner: uid_t, group: gid_t) -> Result<(), Errno>;

    fn clock_getres(clk_id: clockid_t, tp: *mut timespec) -> Result<(), Errno>;

    fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> Result<(), Errno>;

    fn clock_settime(clk_id: clockid_t, tp: *const timespec) -> Result<(), Errno>;

    fn close(fildes: c_int) -> Result<(), Errno>;

    fn dup(fildes: c_int) -> Result<c_int, Errno>;

    fn dup2(fildes: c_int, fildes2: c_int) -> Result<c_int, Errno>;

    unsafe fn execve(
        path: CStr,
        argv: *const *mut c_char,
        envp: *const *mut c_char,
    ) -> Result<(), Errno>;
    unsafe fn fexecve(
        fildes: c_int,
        argv: *const *mut c_char,
        envp: *const *mut c_char,
    ) -> Result<(), Errno>;

    fn exit(status: c_int) -> !;

    fn exit_thread() -> !;

    fn fchdir(fildes: c_int) -> Result<(), Errno>;

    fn fchmod(fildes: c_int, mode: mode_t) -> Result<(), Errno>;

    fn fchown(fildes: c_int, owner: uid_t, group: gid_t) -> Result<(), Errno>;

    fn fdatasync(fildes: c_int) -> Result<(), Errno>;

    fn flock(fd: c_int, operation: c_int) -> Result<(), Errno>;

    fn fstat(fildes: c_int, buf: *mut stat) -> Result<(), Errno>;

    fn fstatvfs(fildes: c_int, buf: *mut statvfs) -> Result<(), Errno>;

    fn fcntl(fildes: c_int, cmd: c_int, arg: c_ulonglong) -> Result<c_int, Errno>;

    fn fork() -> Result<pid_t, Errno>;

    fn fpath(fildes: c_int, out: &mut [u8]) -> Result<ssize_t, Errno>;

    fn fsync(fildes: c_int) -> Result<(), Errno>;

    fn ftruncate(fildes: c_int, length: off_t) -> Result<(), Errno>;

    fn futex(addr: *mut c_int, op: c_int, val: c_int, val2: usize) -> Result<c_long, Errno>;

    fn futimens(fd: c_int, times: *const timespec) -> Result<(), Errno>;

    fn utimens(path: CStr, times: *const timespec) -> Result<(), Errno>;

    fn getcwd(buf: *mut c_char, size: size_t) -> Result<*mut c_char, Errno>;

    fn getdents(fd: c_int, dirents: *mut dirent, bytes: usize) -> Result<c_int, Errno>;

    fn getegid() -> gid_t;

    fn geteuid() -> uid_t;

    fn getgid() -> gid_t;

    unsafe fn getgroups(size: c_int, list: *mut gid_t) -> Result<c_int, Errno>;

    /* Note that this is distinct from the legacy POSIX function
     * getpagesize(), which returns a c_int. On some Linux platforms,
     * page size may be determined through a syscall ("getpagesize"). */
    fn getpagesize() -> usize;

    fn getpgid(pid: pid_t) -> Result<pid_t, Errno>;

    fn getpid() -> pid_t;

    fn getppid() -> pid_t;

    fn getpriority(which: c_int, who: id_t) -> Result<c_int, Errno>;

    fn getrandom(buf: &mut [u8], flags: c_uint) -> Result<ssize_t, Errno>;

    unsafe fn getrlimit(resource: c_int, rlim: *mut rlimit) -> Result<(), Errno>;

    unsafe fn setrlimit(resource: c_int, rlim: *const rlimit) -> Result<(), Errno>;

    fn getsid(pid: pid_t) -> Result<pid_t, Errno>;

    fn gettid() -> pid_t;

    fn gettimeofday(tp: *mut timeval, tzp: *mut timezone) -> Result<(), Errno>;

    fn getuid() -> uid_t;

    fn lchown(path: CStr, owner: uid_t, group: gid_t) -> Result<(), Errno>;

    fn link(path1: CStr, path2: CStr) -> Result<(), Errno>;

    fn lseek(fildes: c_int, offset: off_t, whence: c_int) -> Result<off_t, Errno>;

    fn mkdir(path: CStr, mode: mode_t) -> Result<(), Errno>;

    fn mkfifo(path: CStr, mode: mode_t) -> Result<(), Errno>;

    unsafe fn mlock(addr: *const c_void, len: usize) -> Result<(), Errno>;

    fn mlockall(flags: c_int) -> Result<(), Errno>;

    unsafe fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: c_int,
        flags: c_int,
        fildes: c_int,
        off: off_t,
    ) -> Result<*mut c_void, Errno>;

    unsafe fn mprotect(addr: *mut c_void, len: usize, prot: c_int) -> Result<(), Errno>;

    unsafe fn msync(addr: *mut c_void, len: usize, flags: c_int) -> Result<(), Errno>;

    unsafe fn munlock(addr: *const c_void, len: usize) -> Result<(), Errno>;

    unsafe fn madvise(addr: *mut c_void, len: usize, flags: c_int) -> Result<(), Errno>;

    fn munlockall() -> Result<(), Errno>;

    unsafe fn munmap(addr: *mut c_void, len: usize) -> Result<(), Errno>;

    fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> Result<(), Errno>;

    fn open(path: CStr, oflag: c_int, mode: mode_t) -> Result<c_int, Errno>;

    fn pipe2(fildes: &mut [c_int], flags: c_int) -> Result<(), Errno>;

    unsafe fn rlct_clone(stack: *mut usize) -> Result<crate::pthread::OsTid, Errno>;
    unsafe fn rlct_kill(os_tid: crate::pthread::OsTid, signal: usize) -> Result<(), Errno>;
    fn current_os_tid() -> crate::pthread::OsTid;

    fn read(fildes: c_int, buf: &mut [u8]) -> Result<ssize_t, Errno>;

    fn readlink(pathname: CStr, out: &mut [u8]) -> Result<ssize_t, Errno>;

    fn rename(old: CStr, new: CStr) -> Result<(), Errno>;

    fn rmdir(path: CStr) -> Result<(), Errno>;

    fn sched_yield() -> Result<(), Errno>;

    unsafe fn setgroups(size: size_t, list: *const gid_t) -> Result<(), Errno>;

    fn setpgid(pid: pid_t, pgid: pid_t) -> Result<(), Errno>;

    fn setpriority(which: c_int, who: id_t, prio: c_int) -> Result<(), Errno>;

    fn setregid(rgid: gid_t, egid: gid_t) -> Result<(), Errno>;

    fn setreuid(ruid: uid_t, euid: uid_t) -> Result<(), Errno>;

    fn setsid() -> Result<(), Errno>;

    fn symlink(path1: CStr, path2: CStr) -> Result<(), Errno>;

    fn sync() -> Result<(), Errno>;

    fn umask(mask: mode_t) -> mode_t;

    fn uname(utsname: *mut utsname) -> Result<(), Errno>;

    fn unlink(path: CStr) -> Result<(), Errno>;

    fn waitpid(pid: pid_t, stat_loc: *mut c_int, options: c_int) -> Result<pid_t, Errno>;

    fn write(fildes: c_int, buf: &[u8]) -> Result<ssize_t, Errno>;

    fn verify() -> Result<(), Errno>;
}
