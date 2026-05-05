use core::num::NonZeroU64;

use super::types::*;
use crate::{
    c_str::CStr,
    error::{Errno, Result},
    header::{
        fcntl::{AT_EMPTY_PATH, AT_FDCWD, F_DUPFD},
        signal::sigevent,
        sys_resource::{rlimit, rusage},
        sys_select::timeval,
        sys_stat::stat,
        sys_statvfs::statvfs,
        sys_time::timezone,
        sys_utsname::utsname,
        time::{itimerspec, timespec},
    },
    ld_so::tcb::OsSpecific,
    out::Out,
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
    /// Platform implementation of [`access()`](crate::header::unistd::access) from [`unistd.h`](crate::header::unistd).
    fn access(path: CStr, mode: c_int) -> Result<()> {
        Self::faccessat(AT_FDCWD, path, mode, 0)
    }

    /// Platform implementation of [`faccessat()`](crate::header::unistd::faccessat) from [`unistd.h`](crate::header::unistd).
    fn faccessat(fd: c_int, path: CStr, amode: c_int, flags: c_int) -> Result<()>;

    /// Platform implementation of [`brk()`](crate::header::unistd::brk) from [`unistd.h`](crate::header::unistd).
    unsafe fn brk(addr: *mut c_void) -> Result<*mut c_void>;

    /// Platform implementation of [`chdir()`](crate::header::unistd::chdir) from [`unistd.h`](crate::header::unistd).
    fn chdir(path: CStr) -> Result<()>;

    /// Platform implementation of [`chmod()`](crate::header::sys_stat::chmod) from [`sys/stat.h`](crate::header::sys_stat).
    fn chmod(path: CStr, mode: mode_t) -> Result<()> {
        Self::fchmodat(AT_FDCWD, Some(path), mode, 0)
    }

    /// Platform implementation of [`chown()`](crate::header::unistd::chown) from [`unistd.h`](crate::header::unistd).
    fn chown(path: CStr, owner: uid_t, group: gid_t) -> Result<()> {
        Self::fchownat(AT_FDCWD, path, owner, group, 0)
    }

    /// Platform implementation of [`clock_getres()`](crate::header::time::clock_getres) from [`time.h`](crate::header::time).
    fn clock_getres(clk_id: clockid_t, tp: Option<Out<timespec>>) -> Result<()>;

    // TODO: maybe remove tp and change signature to -> Result<timespec>?
    /// Platform implementation of [`clock_gettime()`](crate::header::time::clock_gettime) from [`time.h`](crate::header::time).
    fn clock_gettime(clk_id: clockid_t, tp: Out<timespec>) -> Result<()>;

    /// Platform implementation of [`clock_settime()`](crate::header::time::clock_settime) from [`time.h`](crate::header::time).
    unsafe fn clock_settime(clk_id: clockid_t, tp: *const timespec) -> Result<()>;

    /// Platform implementation of [`close()`](crate::header::unistd::close) from [`unistd.h`](crate::header::unistd).
    fn close(fildes: c_int) -> Result<()>;

    /// Platform implementation of [`dup()`](crate::header::unistd::dup) from [`unistd.h`](crate::header::unistd).
    fn dup(fildes: c_int) -> Result<c_int> {
        Self::fcntl(fildes, F_DUPFD, 0)
    }

    /// Platform implementation of [`dup2()`](crate::header::unistd::dup2) from [`unistd.h`](crate::header::unistd).
    fn dup2(fildes: c_int, fildes2: c_int) -> Result<c_int>;

    /// Platform implementation of [`execve()`](crate::header::unistd::execve) from [`unistd.h`](crate::header::unistd).
    unsafe fn execve(path: CStr, argv: *const *mut c_char, envp: *const *mut c_char) -> Result<()>;

    /// Platform implementation of [`fexecve()`](crate::header::unistd::fexecve) from [`unistd.h`](crate::header::unistd).
    unsafe fn fexecve(
        fildes: c_int,
        argv: *const *mut c_char,
        envp: *const *mut c_char,
    ) -> Result<()>;

    /// Platform implementation of [`_Exit()`](crate::header::stdlib::_Exit) from [`stdlib.h`](crate::header::stdlib) (or the equivalent [`_exit()`](crate::header::unistd::_exit) from [`unistd.h`](crate::header::unistd)).
    fn exit(status: c_int) -> !;

    unsafe fn exit_thread(stack_base: *mut (), stack_size: usize) -> !;

    /// Platform implementation of [`fchdir()`](crate::header::unistd::fchdir) from [`unistd.h`](crate::header::unistd).
    fn fchdir(fildes: c_int) -> Result<()>;

    /// Platform implementation of [`fchmod()`](crate::header::sys_stat::fchmod) from [`sys/stat.h`](crate::header::sys_stat).
    fn fchmod(fildes: c_int, mode: mode_t) -> Result<()> {
        Self::fchmodat(fildes, Some(c"".into()), mode, AT_EMPTY_PATH)
    }

    /// Platform implementation of [`fchmodat()`](crate::header::sys_stat::fchmodat) from [`sys/stat.h`](crate::header::sys_stat).
    fn fchmodat(dirfd: c_int, path: Option<CStr>, mode: mode_t, flags: c_int) -> Result<()>;

    /// Platform implementation of [`fchown()`](crate::header::unistd::fchown) from [`unistd.h`](crate::header::unistd).
    fn fchown(fildes: c_int, owner: uid_t, group: gid_t) -> Result<()> {
        Self::fchownat(fildes, c"".into(), owner, group, AT_EMPTY_PATH)
    }

    /// Platform implementation of [`fchownat()`](crate::header::unistd::fchownat) from [`unistd.h`](crate::header::unistd).
    fn fchownat(fildes: c_int, path: CStr, owner: uid_t, group: gid_t, flags: c_int) -> Result<()>;

    /// Platform implementation of [`fdatasync()`](crate::header::unistd::fdatasync) from [`unistd.h`](crate::header::unistd).
    fn fdatasync(fildes: c_int) -> Result<()>;

    /// Platform implementation of [`flock()`](crate::header::sys_file::flock) from [`sys/file.h`](crate::header::sys_file).
    fn flock(fd: c_int, operation: c_int) -> Result<()>;

    /// Platform implementation of [`fstat()`](crate::header::sys_stat::fstat) from [`sys/stat.h`](crate::header::sys_stat).
    fn fstat(fildes: c_int, buf: Out<stat>) -> Result<()> {
        Self::fstatat(fildes, Some(c"".into()), buf, 0)
    }

    /// Platform implementation of [`fstatat()`](crate::header::sys_stat::fstatat) from [`sys/stat.h`](crate::header::sys_stat).
    fn fstatat(fildes: c_int, path: Option<CStr>, buf: Out<stat>, flags: c_int) -> Result<()>;

    /// Platform implementation of [`fstatvfs()`](crate::header::sys_statvfs::fstatvfs) from [`sys/statvfs.h`](crate::header::sys_statvfs).
    fn fstatvfs(fildes: c_int, buf: Out<statvfs>) -> Result<()>;

    /// Platform implementation of [`fcntl()`](crate::header::fcntl::fcntl) from [`fcntl.h`](crate::header::fcntl).
    fn fcntl(fildes: c_int, cmd: c_int, arg: c_ulonglong) -> Result<c_int>;

    /// Platform implementation of [`_Fork()`](crate::header::unistd::_Fork) from [`unistd.h`](crate::header::unistd).
    unsafe fn fork() -> Result<pid_t>;

    fn fpath(fildes: c_int, out: &mut [u8]) -> Result<usize>;

    /// Platform implementation of [`fsync()`](crate::header::unistd::fsync) from [`unistd.h`](crate::header::unistd).
    fn fsync(fildes: c_int) -> Result<()>;

    /// Platform implementation of [`ftruncate()`](crate::header::unistd::ftruncate) from [`unistd.h`](crate::header::unistd).
    fn ftruncate(fildes: c_int, length: off_t) -> Result<()>;

    unsafe fn futex_wait(addr: *mut u32, val: u32, deadline: Option<&timespec>) -> Result<()>;

    unsafe fn futex_wake(addr: *mut u32, num: u32) -> Result<u32>;

    /// Platform implementation of [`futimens()`](crate::header::sys_stat::futimens) from [`sys/stat.h`](crate::header::sys_stat).
    unsafe fn futimens(fd: c_int, times: *const timespec) -> Result<()> {
        unsafe { Self::utimensat(fd, c"".into(), times, AT_EMPTY_PATH) }
    }

    /// Platform implementation of [`utimens()`](crate::header::unistd::utimens) (from [`sys/stat.h`](crate::header::sys_stat).
    /// This is an extension of POSIX.
    unsafe fn utimens(path: CStr, times: *const timespec) -> Result<()> {
        unsafe { Self::utimensat(AT_FDCWD, path, times, 0) }
    }

    /// Platform implementation of [`utimensat()`](crate::header::unistd::utimensat) (from [`sys/stat.h`](crate::header::sys_stat).
    unsafe fn utimensat(
        dirfd: c_int,
        path: CStr,
        times: *const timespec,
        flag: c_int,
    ) -> Result<()>;

    /// Platform implementation of [`getcwd()`](crate::header::unistd::getcwd) from [`unistd.h`](crate::header::unistd).
    fn getcwd(buf: Out<[u8]>) -> Result<()>;

    fn getdents(fd: c_int, buf: &mut [u8], opaque_offset: u64) -> Result<usize>;

    fn dir_seek(fd: c_int, opaque_offset: u64) -> Result<()>;

    // SAFETY: This_dent must satisfy platform-specific size and alignment constraints. On Linux,
    // this means the buffer came from a valid getdents64 invocation, whereas on Redox, every
    // possible this_dent slice is safe (and will be validated).
    unsafe fn dent_reclen_offset(this_dent: &[u8], offset: usize) -> Option<(u16, u64)>;

    // Always successful
    /// Platform implementation of [`getegid()`](crate::header::unistd::getegid) from [`unistd.h`](crate::header::unistd).
    fn getegid() -> gid_t;

    // Always successful
    /// Platform implementation of [`geteuid()`](crate::header::unistd::geteuid) from [`unistd.h`](crate::header::unistd).
    fn geteuid() -> uid_t;

    // Always successful
    /// Platform implementation of [`getgid()`](crate::header::unistd::getgid) from [`unistd.h`](crate::header::unistd).
    fn getgid() -> gid_t;

    /// Platform implementation of [`getgroups()`](crate::header::unistd::getgroups) from [`unistd.h`](crate::header::unistd).
    fn getgroups(list: Out<[gid_t]>) -> Result<c_int>;

    /* Note that this is distinct from the legacy POSIX function
     * getpagesize(), which returns a c_int. On some Linux platforms,
     * page size may be determined through a syscall ("getpagesize"). */
    fn getpagesize() -> usize;

    /// Platform implementation of [`getpgid()`](crate::header::unistd::getpgid) from [`unistd.h`](crate::header::unistd).
    fn getpgid(pid: pid_t) -> Result<pid_t>;

    // Always successful
    /// Platform implementation of [`getpid()`](crate::header::unistd::getpid) from [`unistd.h`](crate::header::unistd).
    fn getpid() -> pid_t;

    // Always successful
    /// Platform implementation of [`getppid()`](crate::header::unistd::getppid) from [`unistd.h`](crate::header::unistd).
    fn getppid() -> pid_t;

    /// Platform implementation of [`getpriority()`](crate::header::sys_resource::getpriority) from [`sys/resource.h`](crate::header::sys_resource).
    fn getpriority(which: c_int, who: id_t) -> Result<c_int>;

    /// Platform implementation of [`getrandom()`](crate::header::sys_random::getrandom) from [`sys/random.h`](crate::header::sys_random).
    fn getrandom(buf: &mut [u8], flags: c_uint) -> Result<usize>;

    /// Platform implementation of [`getresgid()`](crate::header::unistd::getresgid) from [`unistd.h`](crate::header::unistd).
    fn getresgid(
        rgid: Option<Out<gid_t>>,
        egid: Option<Out<gid_t>>,
        sgid: Option<Out<gid_t>>,
    ) -> Result<()>;

    /// Platform implementation of [`getresuid()`](crate::header::unistd::getresuid) from [`unistd.h`](crate::header::unistd).
    fn getresuid(
        ruid: Option<Out<uid_t>>,
        euid: Option<Out<uid_t>>,
        suid: Option<Out<uid_t>>,
    ) -> Result<()>;

    /// Platform implementation of [`getrlimit()`](crate::header::sys_resource::getrlimit) from [`sys/resource.h`](crate::header::sys_resource).
    fn getrlimit(resource: c_int, rlim: Out<rlimit>) -> Result<()>;

    /// Platform implementation of [`setrlimit()`](crate::header::sys_resource::setrlimit) from [`sys/resource.h`](crate::header::sys_resource).
    unsafe fn setrlimit(resource: c_int, rlim: *const rlimit) -> Result<()>;

    /// Platform implementation of [`getrusage()`](crate::header::sys_resource::getrusage) from [`sys/resource.h`](crate::header::sys_resource).
    fn getrusage(who: c_int, r_usage: Out<rusage>) -> Result<()>;

    /// Platform implementation of [`getsid()`](crate::header::unistd::getsid) from [`unistd.h`](crate::header::unistd).
    fn getsid(pid: pid_t) -> Result<pid_t>;

    // Always successful
    /// Platform implementation of `gettid()` (TODO) from [`unistd.h`](crate::header::unistd).
    fn gettid() -> pid_t;

    /// Platform implementation of [`gettimeofday()`](crate::header::sys_time::gettimeofday) from [`sys/time.h`](crate::header::sys_time).
    fn gettimeofday(tp: Out<timeval>, tzp: Option<Out<timezone>>) -> Result<()>;

    /// Platform implementation of [`getuid()`](crate::header::unistd::getuid) from [`unistd.h`](crate::header::unistd).
    fn getuid() -> uid_t;

    /// Platform implementation of [`lchown()`](crate::header::unistd::lchown) from [`unistd.h`](crate::header::unistd).
    fn lchown(path: CStr, owner: uid_t, group: gid_t) -> Result<()>;

    /// Platform implementation of [`link()`](crate::header::unistd::link) from [`unistd.h`](crate::header::unistd).
    fn link(path1: CStr, path2: CStr) -> Result<()> {
        Self::linkat(AT_FDCWD, path1, AT_FDCWD, path2, 0)
    }

    /// Platform implementation of [`linkat()`](crate::header::unistd::linkat) from [`unistd.h`](crate::header::unistd).
    fn linkat(fd1: c_int, oldpath: CStr, fd2: c_int, newpath: CStr, flags: c_int) -> Result<()>;

    /// Platform implementation of [`lseek()`](crate::header::unistd::lseek) from [`unistd.h`](crate::header::unistd).
    fn lseek(fildes: c_int, offset: off_t, whence: c_int) -> Result<off_t>;

    /// Platform implementation of [`mkdirat()`](crate::header::sys_stat::mkdirat) from [`sys/stat.h`](crate::header::sys_stat).
    fn mkdirat(fildes: c_int, path: CStr, mode: mode_t) -> Result<()>;

    /// Platform implementation of [`mkdir()`](crate::header::sys_stat::mkdir) from [`sys/stat.h`](crate::header::sys_stat).
    fn mkdir(path: CStr, mode: mode_t) -> Result<()> {
        Self::mkdirat(AT_FDCWD, path, mode)
    }

    /// Platform implementation of [`mkfifoat()`](crate::header::sys_stat::mkfifoat) from [`sys/stat.h`](crate::header::sys_stat).
    fn mkfifoat(dir_fd: c_int, path: CStr, mode: mode_t) -> Result<()>;

    /// Platform implementation of [`mkfifo()`](crate::header::sys_stat::mkfifo) from [`sys/stat.h`](crate::header::sys_stat).
    fn mkfifo(path: CStr, mode: mode_t) -> Result<()> {
        Self::mkfifoat(AT_FDCWD, path, mode)
    }

    /// Platform implementation of [`mknodat()`](crate::header::sys_stat::mknodat) from [`sys/stat.h`](crate::header::sys_stat).
    fn mknodat(fildes: c_int, path: CStr, mode: mode_t, dev: dev_t) -> Result<()>;

    /// Platform implementation of [`mknod()`](crate::header::sys_stat::mknod) from [`sys/stat.h`](crate::header::sys_stat).
    fn mknod(path: CStr, mode: mode_t, dev: dev_t) -> Result<()> {
        Self::mknodat(AT_FDCWD, path, mode, dev)
    }

    /// Platform implementation of [`mlock()`](crate::header::sys_mman::mlock) from [`sys/mman.h`](crate::header::sys_mman).
    unsafe fn mlock(addr: *const c_void, len: usize) -> Result<()>;

    /// Platform implementation of [`mlockall()`](crate::header::sys_mman::mlockall) from [`sys/mman.h`](crate::header::sys_mman).
    unsafe fn mlockall(flags: c_int) -> Result<()>;

    /// Platform implementation of [`mmap()`](crate::header::sys_mman::mmap) from [`sys/mman.h`](crate::header::sys_mman).
    unsafe fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: c_int,
        flags: c_int,
        fildes: c_int,
        off: off_t,
    ) -> Result<*mut c_void>;

    /// Platform implementation of [`mremap()`](crate::header::sys_mman::mremap) from [`sys/mman.h`](crate::header::sys_mman).
    unsafe fn mremap(
        addr: *mut c_void,
        len: usize,
        new_len: usize,
        flags: c_int,
        args: *mut c_void,
    ) -> Result<*mut c_void>;

    /// Platform implementation of [`mprotect()`](crate::header::sys_mman::mprotect) from [`sys/mman.h`](crate::header::sys_mman).
    unsafe fn mprotect(addr: *mut c_void, len: usize, prot: c_int) -> Result<()>;

    /// Platform implementation of [`msync()`](crate::header::sys_mman::msync) from [`sys/mman.h`](crate::header::sys_mman).
    unsafe fn msync(addr: *mut c_void, len: usize, flags: c_int) -> Result<()>;

    /// Platform implementation of [`munlock()`](crate::header::sys_mman::munlock) from [`sys/mman.h`](crate::header::sys_mman).
    unsafe fn munlock(addr: *const c_void, len: usize) -> Result<()>;

    /// Platform implementation of [`madvise()`](crate::header::sys_mman::madvise) from [`sys/mman.h`](crate::header::sys_mman).
    unsafe fn madvise(addr: *mut c_void, len: usize, flags: c_int) -> Result<()>;

    /// Platform implementation of [`munlockall()`](crate::header::sys_mman::munlockall) from [`sys/mman.h`](crate::header::sys_mman).
    unsafe fn munlockall() -> Result<()>;

    /// Platform implementation of [`munmap()`](crate::header::sys_mman::munmap) from [`sys/mman.h`](crate::header::sys_mman).
    unsafe fn munmap(addr: *mut c_void, len: usize) -> Result<()>;

    /// Platform implementation of [`nanosleep()`](crate::header::time::nanosleep) from [`time.h`](crate::header::time).
    unsafe fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> Result<()>;

    /// Platform implementation of [`open()`](crate::header::fcntl::open) from [`fcntl.h`](crate::header::fcntl).
    fn open(path: CStr, oflag: c_int, mode: mode_t) -> Result<c_int> {
        Self::openat(AT_FDCWD, path, oflag, mode)
    }

    /// Platform implementation of `openat()` (TODO) from [`fcntl.h`](crate::header::fcntl).
    fn openat(dirfd: c_int, path: CStr, oflag: c_int, mode: mode_t) -> Result<c_int>;

    /// Platform implementation of [`pipe2()`](crate::header::unistd::pipe2) from [`unistd.h`](crate::header::unistd).
    fn pipe2(fildes: Out<[c_int; 2]>, flags: c_int) -> Result<()>;

    /// Platform implementation of [`posix_fallocate()`](crate::header::fcntl::posix_fallocate) from [`fcntl.h`](crate::header::fcntl).
    fn posix_fallocate(fd: c_int, offset: u64, length: NonZeroU64) -> Result<()>;

    /// Platform implementation of [`posix_getdents()`](crate::header::dirent::posix_getdents) from [`dirent.h`](crate::header::dirent).
    fn posix_getdents(fildes: c_int, buf: &mut [u8]) -> Result<usize>;

    unsafe fn rlct_clone(
        stack: *mut usize,
        os_specific: &mut OsSpecific,
    ) -> Result<pthread::OsTid, Errno>;

    unsafe fn rlct_kill(os_tid: pthread::OsTid, signal: usize) -> Result<()>;

    fn current_os_tid() -> pthread::OsTid;

    /// Platform implementation of [`read()`](crate::header::unistd::read) from [`unistd.h`](crate::header::unistd).
    fn read(fildes: c_int, buf: &mut [u8]) -> Result<usize>;

    /// Platform implementation of [`pread()`](crate::header::unistd::pread) from [`unistd.h`](crate::header::unistd).
    fn pread(fildes: c_int, buf: &mut [u8], offset: off_t) -> Result<usize>;

    /// Platform implementation of [`readlink()`](crate::header::unistd::readlink) from [`unistd.h`](crate::header::unistd).
    fn readlink(pathname: CStr, out: &mut [u8]) -> Result<usize> {
        Self::readlinkat(AT_FDCWD, pathname, out)
    }

    /// Platform implementation of [`readlinkat()`](crate::header::unistd::readlinkat) from [`unistd.h`](crate::header::unistd).
    fn readlinkat(dirfd: c_int, pathname: CStr, out: &mut [u8]) -> Result<usize>;

    /// Platform implementation of [`rename()`](crate::header::stdio::rename) from [`stdio.h`](crate::header::stdio).
    fn rename(old: CStr, new: CStr) -> Result<()> {
        Self::renameat(AT_FDCWD, old, AT_FDCWD, new)
    }

    /// Platform implementation of [`renameat()`](crate::header::stdio::renameat) from [`stdio.h`](crate::header::stdio).
    fn renameat(old_dir: c_int, old_path: CStr, new_dir: c_int, new_path: CStr) -> Result<()> {
        Self::renameat2(old_dir, old_path, new_dir, new_path, 0)
    }

    /// Platform implementation of [`renameat2()`](crate::header::stdio::renameat2) from [`stdio.h`](crate::header::stdio).
    fn renameat2(
        old_dir: c_int,
        old_path: CStr,
        new_dir: c_int,
        new_path: CStr,
        flags: c_uint,
    ) -> Result<()>;

    /// Platform implementation of [`rmdir()`](crate::header::unistd::rmdir) from [`unistd.h`](crate::header::unistd).
    fn rmdir(path: CStr) -> Result<()>;

    /// Platform implementation of [`sched_yield()`](crate::header::sched::sched_yield) from [`sched.h`](crate::header::sched).
    fn sched_yield() -> Result<()>;

    /// Platform implementation of [`setgroups()`](crate::header::grp::setgroups) from [`grp.h`](crate::header::grp).
    unsafe fn setgroups(size: size_t, list: *const gid_t) -> Result<()>;

    /// Platform implementation of [`setpgid()`](crate::header::unistd::setpgid) from [`unistd.h`](crate::header::unistd).
    fn setpgid(pid: pid_t, pgid: pid_t) -> Result<()>;

    /// Platform implementation of [`setpriority()`](crate::header::sys_resource::setpriority) from [`sys/resource.h`](crate::header::sys_resource).
    fn setpriority(which: c_int, who: id_t, prio: c_int) -> Result<()>;

    /// Platform implementation of [`setresgid()`](crate::header::unistd::setresgid) from [`unistd.h`](crate::header::unistd).
    fn setresgid(rgid: gid_t, egid: gid_t, sgid: gid_t) -> Result<()>;

    /// Platform implementation of [`setresuid()`](crate::header::unistd::setresuid) from [`unistd.h`](crate::header::unistd).
    fn setresuid(ruid: uid_t, euid: uid_t, suid: uid_t) -> Result<()>;

    /// Platform implementation of [`setsid()`](crate::header::unistd::setsid) from [`unistd.h`](crate::header::unistd).
    fn setsid() -> Result<c_int>;

    /// Platform implementation of [`symlink()`](crate::header::unistd::symlink) from [`unistd.h`](crate::header::unistd).
    fn symlink(path1: CStr, path2: CStr) -> Result<()> {
        Self::symlinkat(path1, AT_FDCWD, path2)
    }

    /// Platform implementation of [`symlinkat()`](crate::header::unistd::symlinkat) from [`unistd.h`](crate::header::unistd).
    fn symlinkat(path1: CStr, fd: c_int, path2: CStr) -> Result<()>;

    /// Platform implementation of [`sync()`](crate::header::unistd::sync) from [`unistd.h`](crate::header::unistd).
    fn sync() -> Result<()>;

    /// Platform implementation of [`timer_create()`](crate::header::time::timer_create) from [`time.h`](crate::header::time).
    fn timer_create(clock_id: clockid_t, evp: &sigevent, timerid: Out<timer_t>) -> Result<()>;

    /// Platform implementation of [`timer_delete()`](crate::header::time::timer_delete) from [`time.h`](crate::header::time).
    fn timer_delete(timerid: timer_t) -> Result<()>;

    /// Platform implementation of [`timer_gettime()`](crate::header::time::timer_gettime) from [`time.h`](crate::header::time).
    fn timer_gettime(timerid: timer_t, value: Out<itimerspec>) -> Result<()>;

    /// Platform implementation of [`timer_settime()`](crate::header::time::timer_settime) from [`time.h`](crate::header::time).
    fn timer_settime(
        timerid: timer_t,
        flags: c_int,
        value: &itimerspec,
        ovalue: Option<Out<itimerspec>>,
    ) -> Result<()>;

    // Always successful
    /// Platform implementation of [`umask()`](crate::header::sys_stat::umask) from [`sys/stat.h`](crate::header::sys_stat).
    fn umask(mask: mode_t) -> mode_t;

    /// Platform implementation of [`uname()`](crate::header::sys_utsname::uname) from [`sys/utsname.h`](crate::header::sys_utsname).
    fn uname(utsname: Out<utsname>) -> Result<()>;

    /// Platform implementation of [`unlink()`](crate::header::unistd::unlink) from [`unistd.h`](crate::header::unistd).
    fn unlink(path: CStr) -> Result<()> {
        Self::unlinkat(AT_FDCWD, path, 0)
    }

    /// Platform implementation of [`unlinkat()`](crate::header::unistd::unlinkat) from [`unistd.h`](crate::header::unistd).
    fn unlinkat(fd: c_int, path: CStr, flags: c_int) -> Result<()>;

    /// Platform implementation of [`waitpid()`](crate::header::sys_wait::waitpid) from [`sys/wait.h`](crate::header::sys_wait).
    fn waitpid(pid: pid_t, stat_loc: Option<Out<c_int>>, options: c_int) -> Result<pid_t>;

    /// Platform implementation of [`write()`](crate::header::unistd::write) from [`unistd.h`](crate::header::unistd).
    fn write(fildes: c_int, buf: &[u8]) -> Result<usize>;

    /// Platform implementation of [`pwrite()`](crate::header::unistd::pwrite) from [`unistd.h`](crate::header::unistd).
    fn pwrite(fildes: c_int, buf: &[u8], offset: off_t) -> Result<usize>;

    fn verify() -> bool;
}
