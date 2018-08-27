use core::ptr;

use super::types::*;

pub use self::signal::PalSignal;
mod signal;

pub use self::socket::PalSocket;
mod socket;

pub trait Pal {
    fn no_pal(name: &str) -> c_int;

    fn access(path: *const c_char, mode: c_int) -> c_int {
        Self::no_pal("access")
    }

    fn brk(addr: *mut c_void) -> *mut c_void;

    fn chdir(path: *const c_char) -> c_int {
        Self::no_pal("chdir")
    }

    fn chmod(path: *const c_char, mode: mode_t) -> c_int {
        Self::no_pal("chmod")
    }

    fn chown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
        Self::no_pal("chown")
    }

    fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> c_int {
        Self::no_pal("clock_gettime")
    }

    fn close(fildes: c_int) -> c_int {
        Self::no_pal("close")
    }

    fn dup(fildes: c_int) -> c_int {
        Self::no_pal("dup")
    }

    fn dup2(fildes: c_int, fildes2: c_int) -> c_int {
        Self::no_pal("dup2")
    }

    unsafe fn execve(
        path: *const c_char,
        argv: *const *mut c_char,
        envp: *const *mut c_char,
    ) -> c_int {
        Self::no_pal("execve")
    }

    fn exit(status: c_int) -> !;

    fn fchdir(fildes: c_int) -> c_int {
        Self::no_pal("fchdir")
    }

    fn fchmod(fildes: c_int, mode: mode_t) -> c_int {
        Self::no_pal("fchmod")
    }

    fn fchown(fildes: c_int, owner: uid_t, group: gid_t) -> c_int {
        Self::no_pal("fchown")
    }

    fn flock(fd: c_int, operation: c_int) -> c_int {
        Self::no_pal("flock")
    }

    fn fstat(fildes: c_int, buf: *mut stat) -> c_int {
        Self::no_pal("fstat")
    }

    fn fcntl(fildes: c_int, cmd: c_int, arg: c_int) -> c_int {
        Self::no_pal("fcntl")
    }

    fn fork() -> pid_t {
        Self::no_pal("fork")
    }

    fn fsync(fildes: c_int) -> c_int {
        Self::no_pal("fsync")
    }

    fn ftruncate(fildes: c_int, length: off_t) -> c_int {
        Self::no_pal("ftruncate")
    }

    fn futimens(fd: c_int, times: *const timespec) -> c_int {
        Self::no_pal("futimens")
    }

    fn utimens(path: *const c_char, times: *const timespec) -> c_int {
        Self::no_pal("utimens")
    }

    fn getcwd(buf: *mut c_char, size: size_t) -> *mut c_char {
        Self::no_pal("getcwd");
        ptr::null_mut()
    }

    fn getdents(fd: c_int, dirents: *mut dirent, bytes: usize) -> c_int {
        Self::no_pal("getdents")
    }

    fn getegid() -> gid_t {
        Self::no_pal("getegid")
    }

    fn geteuid() -> uid_t {
        Self::no_pal("geteuid")
    }

    fn getgid() -> gid_t {
        Self::no_pal("getgid")
    }

    fn getrusage(who: c_int, r_usage: *mut rusage) -> c_int {
        Self::no_pal("getrusage")
    }

    unsafe fn gethostname(mut name: *mut c_char, len: size_t) -> c_int {
        Self::no_pal("gethostname")
    }

    fn getitimer(which: c_int, out: *mut itimerval) -> c_int {
        Self::no_pal("getitimer")
    }

    fn getpgid(pid: pid_t) -> pid_t {
        Self::no_pal("getpgid")
    }

    fn getpid() -> pid_t {
        Self::no_pal("getpid")
    }

    fn getppid() -> pid_t {
        Self::no_pal("getppid")
    }

    fn gettimeofday(tp: *mut timeval, tzp: *mut timezone) -> c_int {
        Self::no_pal("gettimeofday")
    }

    fn getuid() -> uid_t {
        Self::no_pal("getuid")
    }

    fn ioctl(fd: c_int, request: c_ulong, out: *mut c_void) -> c_int {
        Self::no_pal("ioctl")
    }

    fn isatty(fd: c_int) -> c_int {
        Self::no_pal("isatty")
    }

    fn link(path1: *const c_char, path2: *const c_char) -> c_int {
        Self::no_pal("link")
    }

    fn lseek(fildes: c_int, offset: off_t, whence: c_int) -> off_t {
        Self::no_pal("lseek") as off_t
    }

    fn mkdir(path: *const c_char, mode: mode_t) -> c_int {
        Self::no_pal("mkdir")
    }

    fn mkfifo(path: *const c_char, mode: mode_t) -> c_int {
        Self::no_pal("mkfifo")
    }

    unsafe fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: c_int,
        flags: c_int,
        fildes: c_int,
        off: off_t,
    ) -> *mut c_void {
        Self::no_pal("mmap") as *mut c_void
    }

    unsafe fn munmap(addr: *mut c_void, len: usize) -> c_int {
        Self::no_pal("munmap")
    }

    fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> c_int {
        Self::no_pal("nanosleep")
    }

    fn open(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
        Self::no_pal("open")
    }

    fn pipe(fildes: &mut [c_int]) -> c_int {
        Self::no_pal("pipe")
    }

    fn read(fildes: c_int, buf: &mut [u8]) -> ssize_t {
        Self::no_pal("read") as ssize_t
    }

    fn rename(old: *const c_char, new: *const c_char) -> c_int {
        Self::no_pal("rename")
    }

    fn rmdir(path: *const c_char) -> c_int {
        Self::no_pal("rmdir")
    }

    fn select(
        nfds: c_int,
        readfds: *mut fd_set,
        writefds: *mut fd_set,
        exceptfds: *mut fd_set,
        timeout: *mut timeval,
    ) -> c_int {
        Self::no_pal("select")
    }

    fn setitimer(which: c_int, new: *const itimerval, old: *mut itimerval) -> c_int {
        Self::no_pal("setitimer")
    }

    fn setpgid(pid: pid_t, pgid: pid_t) -> c_int {
        Self::no_pal("setpgid")
    }

    fn setregid(rgid: gid_t, egid: gid_t) -> c_int {
        Self::no_pal("setregid")
    }

    fn setreuid(ruid: uid_t, euid: uid_t) -> c_int {
        Self::no_pal("setreuid")
    }

    fn tcgetattr(fd: c_int, out: *mut termios) -> c_int {
        Self::no_pal("tcgetattr")
    }

    fn tcsetattr(fd: c_int, act: c_int, value: *const termios) -> c_int {
        Self::no_pal("tcsetattr")
    }

    fn times(out: *mut tms) -> clock_t {
        Self::no_pal("times") as clock_t
    }

    fn umask(mask: mode_t) -> mode_t {
        Self::no_pal("umask");
        0
    }

    fn uname(utsname: *mut utsname) -> c_int {
        Self::no_pal("uname")
    }

    fn unlink(path: *const c_char) -> c_int {
        Self::no_pal("unlink")
    }

    fn waitpid(pid: pid_t, stat_loc: *mut c_int, options: c_int) -> pid_t {
        Self::no_pal("waitpid")
    }

    fn write(fildes: c_int, buf: &[u8]) -> ssize_t {
        Self::no_pal("write") as ssize_t
    }
}
