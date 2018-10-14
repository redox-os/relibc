use alloc::vec::Vec;
use core::fmt::Write as _WriteFmt;
use core::{mem, ptr};
use core_io::Write;

use super::types::*;
use super::{errno, FileWriter, Pal};
use c_str::CStr;
use fs::File;
use header::dirent::dirent;
use header::errno::{EINVAL, ENOSYS};
use header::fcntl;
use header::signal::SIGCHLD;
use header::sys_ioctl::{winsize, TCGETS, TCSETS, TIOCGWINSZ};
// use header::sys_resource::rusage;
use header::sys_select::fd_set;
use header::sys_stat::stat;
use header::sys_time::{itimerval, timeval, timezone};
// use header::sys_times::tms;
use header::sys_utsname::utsname;
use header::termios::termios;
use header::time::timespec;

mod signal;
mod socket;

const AT_FDCWD: c_int = -100;
const AT_EMPTY_PATH: c_int = 0x1000;
const AT_REMOVEDIR: c_int = 0x200;

fn e(sys: usize) -> usize {
    if (sys as isize) < 0 && (sys as isize) >= -256 {
        unsafe {
            errno = -(sys as isize) as c_int;
        }
        !0
    } else {
        sys
    }
}

pub struct Sys;

impl Sys {
    fn getitimer(which: c_int, out: *mut itimerval) -> c_int {
        e(unsafe { syscall!(GETITIMER, which, out) }) as c_int
    }

    // fn getrusage(who: c_int, r_usage: *mut rusage) -> c_int {
    //     e(unsafe { syscall!(GETRUSAGE, who, r_usage) }) as c_int
    // }

    pub fn ioctl(fd: c_int, request: c_ulong, out: *mut c_void) -> c_int {
        // TODO: Somehow support varargs to syscall??
        e(unsafe { syscall!(IOCTL, fd, request, out) }) as c_int
    }

    fn setitimer(which: c_int, new: *const itimerval, old: *mut itimerval) -> c_int {
        e(unsafe { syscall!(SETITIMER, which, new, old) }) as c_int
    }

    // fn times(out: *mut tms) -> clock_t {
    //     unsafe { syscall!(TIMES, out) as clock_t }
    // }

    fn umask(mask: mode_t) -> mode_t {
        unsafe { syscall!(UMASK, mask) as mode_t }
    }

    pub fn uname(utsname: *mut utsname) -> c_int {
        e(unsafe { syscall!(UNAME, utsname, 0) }) as c_int
    }
}

impl Pal for Sys {
    fn access(path: &CStr, mode: c_int) -> c_int {
        e(unsafe { syscall!(ACCESS, path.as_ptr(), mode) }) as c_int
    }

    fn brk(addr: *mut c_void) -> *mut c_void {
        unsafe { syscall!(BRK, addr) as *mut c_void }
    }

    fn chdir(path: &CStr) -> c_int {
        e(unsafe { syscall!(CHDIR, path.as_ptr()) }) as c_int
    }

    fn chmod(path: &CStr, mode: mode_t) -> c_int {
        e(unsafe { syscall!(FCHMODAT, AT_FDCWD, path.as_ptr(), mode, 0) }) as c_int
    }

    fn chown(path: &CStr, owner: uid_t, group: gid_t) -> c_int {
        e(unsafe {
            syscall!(
                FCHOWNAT,
                AT_FDCWD,
                path.as_ptr(),
                owner as u32,
                group as u32
            )
        }) as c_int
    }

    fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> c_int {
        e(unsafe { syscall!(CLOCK_GETTIME, clk_id, tp) }) as c_int
    }

    fn close(fildes: c_int) -> c_int {
        e(unsafe { syscall!(CLOSE, fildes) }) as c_int
    }

    fn dup(fildes: c_int) -> c_int {
        e(unsafe { syscall!(DUP, fildes) }) as c_int
    }

    fn dup2(fildes: c_int, fildes2: c_int) -> c_int {
        e(unsafe { syscall!(DUP3, fildes, fildes2, 0) }) as c_int
    }

    unsafe fn execve(path: &CStr, argv: *const *mut c_char, envp: *const *mut c_char) -> c_int {
        e(syscall!(EXECVE, path.as_ptr(), argv, envp)) as c_int
    }

    fn exit(status: c_int) -> ! {
        unsafe {
            syscall!(EXIT, status);
        }
        loop {}
    }

    fn fchdir(fildes: c_int) -> c_int {
        e(unsafe { syscall!(FCHDIR, fildes) }) as c_int
    }

    fn fchmod(fildes: c_int, mode: mode_t) -> c_int {
        e(unsafe { syscall!(FCHMOD, fildes, mode) }) as c_int
    }

    fn fchown(fildes: c_int, owner: uid_t, group: gid_t) -> c_int {
        e(unsafe { syscall!(FCHOWN, fildes, owner, group) }) as c_int
    }

    fn flock(fd: c_int, operation: c_int) -> c_int {
        e(unsafe { syscall!(FLOCK, fd, operation) }) as c_int
    }

    fn fstat(fildes: c_int, buf: *mut stat) -> c_int {
        let empty = b"\0";
        let empty_ptr = empty.as_ptr() as *const c_char;
        e(unsafe { syscall!(NEWFSTATAT, fildes, empty_ptr, buf, AT_EMPTY_PATH) }) as c_int
    }

    fn fcntl(fildes: c_int, cmd: c_int, arg: c_int) -> c_int {
        e(unsafe { syscall!(FCNTL, fildes, cmd, arg) }) as c_int
    }

    fn fork() -> pid_t {
        e(unsafe { syscall!(CLONE, SIGCHLD, 0) }) as pid_t
    }

    fn fsync(fildes: c_int) -> c_int {
        e(unsafe { syscall!(FSYNC, fildes) }) as c_int
    }

    fn ftruncate(fildes: c_int, length: off_t) -> c_int {
        e(unsafe { syscall!(FTRUNCATE, fildes, length) }) as c_int
    }

    fn futex(addr: *mut c_int, op: c_int, val: c_int) -> c_int {
        unsafe { syscall!(FUTEX, addr, op, val, 0, 0, 0) as c_int }
    }

    fn futimens(fd: c_int, times: *const timespec) -> c_int {
        e(unsafe { syscall!(UTIMENSAT, fd, ptr::null::<c_char>(), times, 0) }) as c_int
    }

    fn utimens(path: &CStr, times: *const timespec) -> c_int {
        e(unsafe { syscall!(UTIMENSAT, AT_FDCWD, path.as_ptr(), times, 0) }) as c_int
    }

    fn getcwd(buf: *mut c_char, size: size_t) -> *mut c_char {
        if e(unsafe { syscall!(GETCWD, buf, size) }) == !0 {
            ptr::null_mut()
        } else {
            buf
        }
    }

    fn getdents(fd: c_int, dirents: *mut dirent, bytes: usize) -> c_int {
        unsafe { syscall!(GETDENTS64, fd, dirents, bytes) as c_int }
    }

    fn getegid() -> gid_t {
        e(unsafe { syscall!(GETEGID) }) as gid_t
    }

    fn geteuid() -> uid_t {
        e(unsafe { syscall!(GETEUID) }) as uid_t
    }

    fn getgid() -> gid_t {
        e(unsafe { syscall!(GETGID) }) as gid_t
    }

    fn gethostname(mut name: *mut c_char, mut len: size_t) -> c_int {
        unsafe {
            let mut uts = mem::uninitialized();
            let err = Sys::uname(&mut uts);
            if err < 0 {
                mem::forget(uts);
                return err;
            }
            for c in uts.nodename.iter() {
                if len == 0 {
                    break;
                }
                len -= 1;

                *name = *c;

                if *name == 0 {
                    // We do want to copy the zero also, so we check this after the copying.
                    break;
                }

                name = name.offset(1);
            }
            0
        }
    }

    fn getpgid(pid: pid_t) -> pid_t {
        e(unsafe { syscall!(GETPGID, pid) }) as pid_t
    }

    fn getpid() -> pid_t {
        e(unsafe { syscall!(GETPID) }) as pid_t
    }

    fn getppid() -> pid_t {
        e(unsafe { syscall!(GETPPID) }) as pid_t
    }

    fn gettimeofday(tp: *mut timeval, tzp: *mut timezone) -> c_int {
        e(unsafe { syscall!(GETTIMEOFDAY, tp, tzp) }) as c_int
    }

    fn getuid() -> uid_t {
        e(unsafe { syscall!(GETUID) }) as uid_t
    }

    fn isatty(fd: c_int) -> c_int {
        let mut winsize = winsize::default();
        (Self::ioctl(fd, TIOCGWINSZ, &mut winsize as *mut _ as *mut c_void) == 0) as c_int
    }

    fn link(path1: &CStr, path2: &CStr) -> c_int {
        e(unsafe {
            syscall!(
                LINKAT,
                AT_FDCWD,
                path1.as_ptr(),
                AT_FDCWD,
                path2.as_ptr(),
                0
            )
        }) as c_int
    }

    fn lseek(fildes: c_int, offset: off_t, whence: c_int) -> off_t {
        e(unsafe { syscall!(LSEEK, fildes, offset, whence) }) as off_t
    }

    fn mkdir(path: &CStr, mode: mode_t) -> c_int {
        e(unsafe { syscall!(MKDIRAT, AT_FDCWD, path.as_ptr(), mode) }) as c_int
    }

    fn mkfifo(path: &CStr, mode: mode_t) -> c_int {
        e(unsafe { syscall!(MKNODAT, AT_FDCWD, path.as_ptr(), mode, 0) }) as c_int
    }

    unsafe fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: c_int,
        flags: c_int,
        fildes: c_int,
        off: off_t,
    ) -> *mut c_void {
        e(syscall!(MMAP, addr, len, prot, flags, fildes, off)) as *mut c_void
    }

    unsafe fn munmap(addr: *mut c_void, len: usize) -> c_int {
        e(syscall!(MUNMAP, addr, len)) as c_int
    }

    fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> c_int {
        e(unsafe { syscall!(NANOSLEEP, rqtp, rmtp) }) as c_int
    }

    fn open(path: &CStr, oflag: c_int, mode: mode_t) -> c_int {
        e(unsafe { syscall!(OPENAT, AT_FDCWD, path.as_ptr(), oflag, mode) }) as c_int
    }

    fn pipe(fildes: &mut [c_int]) -> c_int {
        e(unsafe { syscall!(PIPE2, fildes.as_mut_ptr(), 0) }) as c_int
    }

    fn read(fildes: c_int, buf: &mut [u8]) -> ssize_t {
        e(unsafe { syscall!(READ, fildes, buf.as_mut_ptr(), buf.len()) }) as ssize_t
    }

    fn realpath(pathname: &CStr, out: &mut [u8]) -> c_int {
        fn readlink(pathname: &CStr, out: &mut [u8]) -> ssize_t {
            e(unsafe { syscall!(READLINKAT, AT_FDCWD, pathname.as_ptr(), out.as_mut_ptr(), out.len()) }) as ssize_t
        }

        let file = match File::open(pathname, fcntl::O_PATH) {
            Ok(file) => file,
            Err(_) => return -1
        };

        if out.is_empty() {
            return 0;
        }

        let mut proc_path = b"/proc/self/fd/".to_vec();
        write!(proc_path, "{}", *file).unwrap();
        proc_path.push(0);

        let len = out.len();
        let read = readlink(CStr::from_bytes_with_nul(&proc_path).unwrap(), &mut out[..len-1]);
        if read < 0 {
            return -1;
        }
        out[read as usize] = 0;

        // TODO: Should these checks from musl be ported?
        // https://gitlab.com/bminor/musl/blob/master/src/misc/realpath.c#L33-38
        // I'm not exactly sure what they're checking...
        // Seems to be a sanity check whether or not it's still the same file?

        0
    }

    fn rename(old: &CStr, new: &CStr) -> c_int {
        e(unsafe { syscall!(RENAMEAT, AT_FDCWD, old.as_ptr(), AT_FDCWD, new.as_ptr()) }) as c_int
    }

    fn rmdir(path: &CStr) -> c_int {
        e(unsafe { syscall!(UNLINKAT, AT_FDCWD, path.as_ptr(), AT_REMOVEDIR) }) as c_int
    }

    fn select(
        nfds: c_int,
        readfds: *mut fd_set,
        writefds: *mut fd_set,
        exceptfds: *mut fd_set,
        timeout: *mut timeval,
    ) -> c_int {
        e(unsafe { syscall!(SELECT, nfds, readfds, writefds, exceptfds, timeout) }) as c_int
    }

    fn setpgid(pid: pid_t, pgid: pid_t) -> c_int {
        e(unsafe { syscall!(SETPGID, pid, pgid) }) as c_int
    }

    fn setregid(rgid: gid_t, egid: gid_t) -> c_int {
        e(unsafe { syscall!(SETREGID, rgid, egid) }) as c_int
    }

    fn setreuid(ruid: uid_t, euid: uid_t) -> c_int {
        e(unsafe { syscall!(SETREUID, ruid, euid) }) as c_int
    }

    fn tcgetattr(fd: c_int, out: *mut termios) -> c_int {
        Self::ioctl(fd, TCGETS, out as *mut c_void)
    }

    fn tcsetattr(fd: c_int, act: c_int, value: *const termios) -> c_int {
        if act < 0 || act > 2 {
            unsafe {
                errno = EINVAL;
            }
            return -1;
        }
        // This is safe because ioctl shouldn't modify the value
        Self::ioctl(fd, TCSETS + act as c_ulong, value as *mut c_void)
    }

    fn unlink(path: &CStr) -> c_int {
        e(unsafe { syscall!(UNLINKAT, AT_FDCWD, path.as_ptr(), 0) }) as c_int
    }

    fn waitpid(pid: pid_t, stat_loc: *mut c_int, options: c_int) -> pid_t {
        e(unsafe { syscall!(WAIT4, pid, stat_loc, options, 0) }) as pid_t
    }

    fn write(fildes: c_int, buf: &[u8]) -> ssize_t {
        e(unsafe { syscall!(WRITE, fildes, buf.as_ptr(), buf.len()) }) as ssize_t
    }
}
