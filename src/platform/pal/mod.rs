use super::types::*;
use crate::{
    c_str::CStr,
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
    fn access(path: CStr, mode: c_int) -> c_int;

    fn brk(addr: *mut c_void) -> *mut c_void;

    fn chdir(path: CStr) -> c_int;

    fn chmod(path: CStr, mode: mode_t) -> c_int;

    fn chown(path: CStr, owner: uid_t, group: gid_t) -> c_int;

    fn clock_getres(clk_id: clockid_t, tp: *mut timespec) -> c_int;

    fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> c_int;

    fn clock_settime(clk_id: clockid_t, tp: *const timespec) -> c_int;

    fn close(fildes: c_int) -> c_int;

    fn dup(fildes: c_int) -> c_int;

    fn dup2(fildes: c_int, fildes2: c_int) -> c_int;

    unsafe fn execve(path: CStr, argv: *const *mut c_char, envp: *const *mut c_char) -> c_int;
    unsafe fn fexecve(fildes: c_int, argv: *const *mut c_char, envp: *const *mut c_char) -> c_int;

    fn exit(status: c_int) -> !;

    fn exit_thread() -> !;

    fn fchdir(fildes: c_int) -> c_int;

    fn fchmod(fildes: c_int, mode: mode_t) -> c_int;

    fn fchown(fildes: c_int, owner: uid_t, group: gid_t) -> c_int;

    fn fdatasync(fildes: c_int) -> c_int;

    fn flock(fd: c_int, operation: c_int) -> c_int;

    fn fstat(fildes: c_int, buf: *mut stat) -> c_int;

    fn fstatvfs(fildes: c_int, buf: *mut statvfs) -> c_int;

    fn fcntl(fildes: c_int, cmd: c_int, arg: c_ulonglong) -> c_int;

    fn fork() -> pid_t;

    fn fpath(fildes: c_int, out: &mut [u8]) -> ssize_t;

    fn fsync(fildes: c_int) -> c_int;

    fn ftruncate(fildes: c_int, length: off_t) -> c_int;

    fn futex(
        addr: *mut c_int,
        op: c_int,
        val: c_int,
        val2: usize,
    ) -> Result<c_long, crate::pthread::Errno>;

    fn futimens(fd: c_int, times: *const timespec) -> c_int;

    fn utimens(path: CStr, times: *const timespec) -> c_int;

    fn getcwd(buf: *mut c_char, size: size_t) -> *mut c_char;

    fn getdents(fd: c_int, dirents: *mut dirent, bytes: usize) -> c_int;

    fn getegid() -> gid_t;

    fn geteuid() -> uid_t;

    fn getgid() -> gid_t;

    unsafe fn getgroups(size: c_int, list: *mut gid_t) -> c_int;

    /* Note that this is distinct from the legacy POSIX function
     * getpagesize(), which returns a c_int. On some Linux platforms,
     * page size may be determined through a syscall ("getpagesize"). */
    fn getpagesize() -> usize;

    fn getpgid(pid: pid_t) -> pid_t;

    fn getpid() -> pid_t;

    fn getppid() -> pid_t;

    fn getpriority(which: c_int, who: id_t) -> c_int;

    fn getrandom(buf: &mut [u8], flags: c_uint) -> ssize_t;

    unsafe fn getrlimit(resource: c_int, rlim: *mut rlimit) -> c_int;

    unsafe fn setrlimit(resource: c_int, rlim: *const rlimit) -> c_int;

    fn getsid(pid: pid_t) -> pid_t;

    fn gettid() -> pid_t;

    fn gettimeofday(tp: *mut timeval, tzp: *mut timezone) -> c_int;

    fn getuid() -> uid_t;

    fn lchown(path: CStr, owner: uid_t, group: gid_t) -> c_int;

    fn link(path1: CStr, path2: CStr) -> c_int;

    fn lseek(fildes: c_int, offset: off_t, whence: c_int) -> off_t;

    fn mkdir(path: CStr, mode: mode_t) -> c_int;

    fn mkfifo(path: CStr, mode: mode_t) -> c_int;

    fn mknodat(fildes: c_int, path: CStr, mode: mode_t, dev: dev_t) -> c_int;

    fn mknod(path: CStr, mode: mode_t, dev: dev_t) -> c_int;

    unsafe fn mlock(addr: *const c_void, len: usize) -> c_int;

    fn mlockall(flags: c_int) -> c_int;

    unsafe fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: c_int,
        flags: c_int,
        fildes: c_int,
        off: off_t,
    ) -> *mut c_void;

    unsafe fn mremap(
        addr: *mut c_void,
        len: usize,
        new_len: usize,
        flags: c_int,
        args: *mut c_void,
    ) -> *mut c_void;

    unsafe fn mprotect(addr: *mut c_void, len: usize, prot: c_int) -> c_int;

    unsafe fn msync(addr: *mut c_void, len: usize, flags: c_int) -> c_int;

    unsafe fn munlock(addr: *const c_void, len: usize) -> c_int;

    unsafe fn madvise(addr: *mut c_void, len: usize, flags: c_int) -> c_int;

    fn munlockall() -> c_int;

    unsafe fn munmap(addr: *mut c_void, len: usize) -> c_int;

    fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> c_int;

    fn open(path: CStr, oflag: c_int, mode: mode_t) -> c_int;

    fn pipe2(fildes: &mut [c_int], flags: c_int) -> c_int;

    unsafe fn rlct_clone(stack: *mut usize)
        -> Result<crate::pthread::OsTid, crate::pthread::Errno>;
    unsafe fn rlct_kill(
        os_tid: crate::pthread::OsTid,
        signal: usize,
    ) -> Result<(), crate::pthread::Errno>;
    fn current_os_tid() -> crate::pthread::OsTid;

    fn read(fildes: c_int, buf: &mut [u8]) -> ssize_t;

    fn readlink(pathname: CStr, out: &mut [u8]) -> ssize_t;

    fn rename(old: CStr, new: CStr) -> c_int;

    fn rmdir(path: CStr) -> c_int;

    fn sched_yield() -> c_int;

    unsafe fn setgroups(size: size_t, list: *const gid_t) -> c_int;

    fn setpgid(pid: pid_t, pgid: pid_t) -> c_int;

    fn setpriority(which: c_int, who: id_t, prio: c_int) -> c_int;

    fn setregid(rgid: gid_t, egid: gid_t) -> c_int;

    fn setreuid(ruid: uid_t, euid: uid_t) -> c_int;

    fn setsid() -> c_int;

    fn symlink(path1: CStr, path2: CStr) -> c_int;

    fn sync() -> c_int;

    fn umask(mask: mode_t) -> mode_t;

    fn uname(utsname: *mut utsname) -> c_int;

    fn unlink(path: CStr) -> c_int;

    fn waitpid(pid: pid_t, stat_loc: *mut c_int, options: c_int) -> pid_t;

    fn write(fildes: c_int, buf: &[u8]) -> ssize_t;

    fn verify() -> bool;
}
