use super::types::*;
use crate::{
    c_str::CStr,
    error::{Errno, Result},
    header::{
        sys_resource::{rlimit, rusage},
        sys_stat::stat,
        sys_statvfs::statvfs,
        sys_time::{timeval, timezone},
        sys_utsname::utsname,
        time::timespec,
    },
    pthread,
};

pub use self::epoll::PalEpoll;
mod epoll;

pub use self::ptrace::PalPtrace;
mod ptrace;

pub use self::signal::PalSignal;
mod signal;

pub use self::socket::PalSocket;
mod socket;

/// Platform abstraction layer, a platform-agnostic abstraction over syscalls.
pub trait Pal {
    fn access(path: CStr, mode: c_int) -> Result<()>;

    unsafe fn brk(addr: *mut c_void) -> Result<*mut c_void>;

    fn chdir(path: CStr) -> Result<()>;

    fn set_default_scheme(scheme: CStr) -> Result<(), Errno>;

    fn chmod(path: CStr, mode: mode_t) -> Result<()>;

    fn chown(path: CStr, owner: uid_t, group: gid_t) -> Result<()>;

    unsafe fn clock_getres(clk_id: clockid_t, tp: *mut timespec) -> Result<()>;

    unsafe fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> Result<()>;

    unsafe fn clock_settime(clk_id: clockid_t, tp: *const timespec) -> Result<()>;

    fn close(fildes: c_int) -> Result<()>;

    fn dup(fildes: c_int) -> Result<c_int>;

    fn dup2(fildes: c_int, fildes2: c_int) -> Result<c_int>;

    unsafe fn execve(path: CStr, argv: *const *mut c_char, envp: *const *mut c_char) -> Result<()>;
    unsafe fn fexecve(
        fildes: c_int,
        argv: *const *mut c_char,
        envp: *const *mut c_char,
    ) -> Result<()>;

    fn exit(status: c_int) -> !;

    unsafe fn exit_thread(stack_base: *mut (), stack_size: usize) -> !;

    fn fchdir(fildes: c_int) -> Result<()>;

    fn fchmod(fildes: c_int, mode: mode_t) -> Result<()>;

    fn fchown(fildes: c_int, owner: uid_t, group: gid_t) -> Result<()>;

    fn fdatasync(fildes: c_int) -> Result<()>;

    fn flock(fd: c_int, operation: c_int) -> Result<()>;

    unsafe fn fstat(fildes: c_int, buf: *mut stat) -> Result<()>;

    unsafe fn fstatvfs(fildes: c_int, buf: *mut statvfs) -> Result<()>;

    fn fcntl(fildes: c_int, cmd: c_int, arg: c_ulonglong) -> Result<c_int>;

    unsafe fn fork() -> Result<pid_t>;

    fn fpath(fildes: c_int, out: &mut [u8]) -> Result<usize>;

    fn fsync(fildes: c_int) -> Result<()>;

    fn ftruncate(fildes: c_int, length: off_t) -> Result<()>;

    unsafe fn futex_wait(addr: *mut u32, val: u32, deadline: Option<&timespec>) -> Result<()>;
    unsafe fn futex_wake(addr: *mut u32, num: u32) -> Result<u32>;

    unsafe fn futimens(fd: c_int, times: *const timespec) -> Result<()>;

    unsafe fn utimens(path: CStr, times: *const timespec) -> Result<()>;

    unsafe fn getcwd(buf: *mut c_char, size: size_t) -> Result<()>;

    fn getdents(fd: c_int, buf: &mut [u8], opaque_offset: u64) -> Result<usize>;
    fn dir_seek(fd: c_int, opaque_offset: u64) -> Result<()>;

    // SAFETY: This_dent must satisfy platform-specific size and alignment constraints. On Linux,
    // this means the buffer came from a valid getdents64 invocation, whereas on Redox, every
    // possible this_dent slice is safe (and will be validated).
    unsafe fn dent_reclen_offset(this_dent: &[u8], offset: usize) -> Option<(u16, u64)>;

    // Always successful
    fn getegid() -> gid_t;

    // Always successful
    fn geteuid() -> uid_t;

    // Always successful
    fn getgid() -> gid_t;

    unsafe fn getgroups(size: c_int, list: *mut gid_t) -> Result<c_int>;

    /* Note that this is distinct from the legacy POSIX function
     * getpagesize(), which returns a c_int. On some Linux platforms,
     * page size may be determined through a syscall ("getpagesize"). */
    fn getpagesize() -> usize;

    fn getpgid(pid: pid_t) -> Result<pid_t>;

    // Always successful
    fn getpid() -> pid_t;

    // Always successful
    fn getppid() -> pid_t;

    fn getpriority(which: c_int, who: id_t) -> Result<c_int>;

    fn getrandom(buf: &mut [u8], flags: c_uint) -> Result<usize>;

    unsafe fn getrlimit(resource: c_int, rlim: *mut rlimit) -> Result<()>;

    unsafe fn setrlimit(resource: c_int, rlim: *const rlimit) -> Result<()>;

    fn getrusage(who: c_int, r_usage: &mut rusage) -> Result<()>;

    fn getsid(pid: pid_t) -> Result<pid_t>;

    // Always successful
    fn gettid() -> pid_t;

    unsafe fn gettimeofday(tp: *mut timeval, tzp: *mut timezone) -> Result<()>;

    fn getuid() -> uid_t;

    fn lchown(path: CStr, owner: uid_t, group: gid_t) -> Result<()>;

    fn link(path1: CStr, path2: CStr) -> Result<()>;

    fn lseek(fildes: c_int, offset: off_t, whence: c_int) -> Result<off_t>;

    fn mkdir(path: CStr, mode: mode_t) -> Result<()>;

    fn mkfifo(path: CStr, mode: mode_t) -> Result<()>;

    fn mknodat(fildes: c_int, path: CStr, mode: mode_t, dev: dev_t) -> Result<()>;

    fn mknod(path: CStr, mode: mode_t, dev: dev_t) -> Result<()>;

    unsafe fn mlock(addr: *const c_void, len: usize) -> Result<()>;

    unsafe fn mlockall(flags: c_int) -> Result<()>;

    unsafe fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: c_int,
        flags: c_int,
        fildes: c_int,
        off: off_t,
    ) -> Result<*mut c_void>;

    unsafe fn mremap(
        addr: *mut c_void,
        len: usize,
        new_len: usize,
        flags: c_int,
        args: *mut c_void,
    ) -> Result<*mut c_void>;

    unsafe fn mprotect(addr: *mut c_void, len: usize, prot: c_int) -> Result<()>;

    unsafe fn msync(addr: *mut c_void, len: usize, flags: c_int) -> Result<()>;

    unsafe fn munlock(addr: *const c_void, len: usize) -> Result<()>;

    unsafe fn madvise(addr: *mut c_void, len: usize, flags: c_int) -> Result<()>;

    unsafe fn munlockall() -> Result<()>;

    unsafe fn munmap(addr: *mut c_void, len: usize) -> Result<()>;

    unsafe fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> Result<()>;

    fn open(path: CStr, oflag: c_int, mode: mode_t) -> Result<c_int>;

    fn pipe2(fildes: &mut [c_int], flags: c_int) -> Result<()>;

    unsafe fn rlct_clone(stack: *mut usize) -> Result<pthread::OsTid, Errno>;
    unsafe fn rlct_kill(os_tid: pthread::OsTid, signal: usize) -> Result<()>;

    fn current_os_tid() -> pthread::OsTid;

    fn read(fildes: c_int, buf: &mut [u8]) -> Result<usize>;
    fn pread(fildes: c_int, buf: &mut [u8], offset: off_t) -> Result<usize>;

    fn readlink(pathname: CStr, out: &mut [u8]) -> Result<usize>;

    fn rename(old: CStr, new: CStr) -> Result<()>;

    fn rmdir(path: CStr) -> Result<()>;

    fn sched_yield() -> Result<()>;

    unsafe fn setgroups(size: size_t, list: *const gid_t) -> Result<()>;

    fn setpgid(pid: pid_t, pgid: pid_t) -> Result<()>;

    fn setpriority(which: c_int, who: id_t, prio: c_int) -> Result<()>;

    fn setresgid(rgid: gid_t, egid: gid_t, sgid: gid_t) -> Result<()>;

    fn setresuid(ruid: uid_t, euid: uid_t, suid: uid_t) -> Result<()>;

    fn setsid() -> Result<()>;

    fn symlink(path1: CStr, path2: CStr) -> Result<()>;

    fn sync() -> Result<()>;

    // Always successful
    fn umask(mask: mode_t) -> mode_t;

    unsafe fn uname(utsname: *mut utsname) -> Result<()>;

    fn unlink(path: CStr) -> Result<()>;

    unsafe fn waitpid(pid: pid_t, stat_loc: *mut c_int, options: c_int) -> Result<pid_t>;

    fn write(fildes: c_int, buf: &[u8]) -> Result<usize>;
    fn pwrite(fildes: c_int, buf: &[u8], offset: off_t) -> Result<usize>;

    fn verify() -> bool;
}
