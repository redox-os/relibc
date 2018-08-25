use types::*;

pub trait Pal {
    fn no_pal<T>(name: &str) -> T;

    unsafe fn accept(socket: c_int, address: *mut sockaddr, address_len: *mut socklen_t) -> c_int {
        Self::no_pal("accept")
    }

    fn access(path: *const c_char, mode: c_int) -> c_int {
        Self::no_pal("access")
    }

    unsafe fn bind(socket: c_int, address: *const sockaddr, address_len: socklen_t) -> c_int {
        Self::no_pal("bind")
    }

    fn brk(addr: *mut c_void) -> *mut c_void {
        Self::no_pal("brk")
    }

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

    unsafe fn connect(socket: c_int, address: *const sockaddr, address_len: socklen_t) -> c_int {
        Self::no_pal("connect")
    }

    fn dup(fildes: c_int) -> c_int {
        Self::no_pal("dup")
    }

    fn dup2(fildes: c_int, fildes2: c_int) -> c_int {
        Self::no_pal("dup2")
    }

    fn execve(path: *const c_char, argv: *const *mut c_char, envp: *const *mut c_char) -> c_int {
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
        Self::no_pal("getcwd")
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

    unsafe fn getpeername(socket: c_int, address: *mut sockaddr, address_len: *mut socklen_t) -> c_int {
        Self::no_pal("getpeername")
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

    unsafe fn getsockname(socket: c_int, address: *mut sockaddr, address_len: *mut socklen_t) -> c_int {
        Self::no_pal("getsockname")
    }

    fn getsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: *mut c_void,
        option_len: *mut socklen_t,
    ) -> c_int {
        Self::no_pal("getsockopt")
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

    fn kill(pid: pid_t, sig: c_int) -> c_int {
        Self::no_pal("kill")
    }

    fn killpg(pgrp: pid_t, sig: c_int) -> c_int {
        Self::no_pal("killpg")
    }

    fn link(path1: *const c_char, path2: *const c_char) -> c_int {
        Self::no_pal("link")
    }

    fn listen(socket: c_int, backlog: c_int) -> c_int {
        Self::no_pal("listen")
    }

    fn lseek(fildes: c_int, offset: off_t, whence: c_int) -> off_t {
        Self::no_pal("lseek")
    }

    fn lstat(file: *const c_char, buf: *mut stat) -> c_int {
        Self::no_pal("lstat")
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
        Self::no_pal("mmap")
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

    fn raise(sig: c_int) -> c_int {
        Self::no_pal("raise")
    }

    fn read(fildes: c_int, buf: &mut [u8]) -> ssize_t {
        Self::no_pal("read")
    }

    unsafe fn recvfrom(
        socket: c_int,
        buf: *mut c_void,
        len: size_t,
        flags: c_int,
        address: *mut sockaddr,
        address_len: *mut socklen_t,
    ) -> ssize_t {
        Self::no_pal("recvfrom")
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

    unsafe fn sendto(
        socket: c_int,
        buf: *const c_void,
        len: size_t,
        flags: c_int,
        dest_addr: *const sockaddr,
        dest_len: socklen_t,
    ) -> ssize_t {
        Self::no_pal("sendto")
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

    fn setsockopt(
        socket: c_int,
        level: c_int,
        option_name: c_int,
        option_value: *const c_void,
        option_len: socklen_t,
    ) -> c_int {
        Self::no_pal("setsockopt")
    }

    fn shutdown(socket: c_int, how: c_int) -> c_int {
        Self::no_pal("shutdown")
    }

    unsafe fn sigaction(sig: c_int, act: *const sigaction, oact: *mut sigaction) -> c_int {
        Self::no_pal("sigaction")
    }

    fn sigprocmask(how: c_int, set: *const sigset_t, oset: *mut sigset_t) -> c_int {
        Self::no_pal("sigprocmask")
    }

    fn stat(file: *const c_char, buf: *mut stat) -> c_int {
        Self::no_pal("stat")
    }

    fn socket(domain: c_int, kind: c_int, protocol: c_int) -> c_int {
        Self::no_pal("socket")
    }

    fn socketpair(domain: c_int, kind: c_int, protocol: c_int, socket_vector: *mut c_int) -> c_int {
        Self::no_pal("socketpair")
    }

    fn tcgetattr(fd: c_int, out: *mut termios) -> c_int {
        Self::no_pal("tcgetattr")
    }

    fn tcsetattr(fd: c_int, act: c_int, value: *const termios) -> c_int {
        Self::no_pal("tcsetattr")
    }

    fn times(out: *mut tms) -> clock_t {
        Self::no_pal("times")
    }

    fn umask(mask: mode_t) -> mode_t {
        Self::no_pal("umask")
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
        Self::no_pal("write")
    }
}
