use crate::io::Write;
use core::{arch::asm, ptr};

use super::{errno, types::*, Pal};
use crate::{
    c_str::CStr,
    header::{dirent::dirent, errno::EINVAL, signal::SIGCHLD, sys_stat::S_IFIFO},
};
// use header::sys_resource::rusage;
use crate::header::{
    sys_resource::rlimit,
    sys_stat::stat,
    sys_statvfs::statvfs,
    sys_time::{timeval, timezone},
};
// use header::sys_times::tms;
use crate::header::{sys_utsname::utsname, time::timespec};

mod epoll;
mod ptrace;
mod signal;
mod socket;

const AT_FDCWD: c_int = -100;
const AT_EMPTY_PATH: c_int = 0x1000;
const AT_REMOVEDIR: c_int = 0x200;

const SYS_CLONE: usize = 56;
const CLONE_VM: usize = 0x0100;
const CLONE_FS: usize = 0x0200;
const CLONE_FILES: usize = 0x0400;
const CLONE_SIGHAND: usize = 0x0800;
const CLONE_THREAD: usize = 0x00010000;

#[repr(C)]
#[derive(Default)]
struct linux_statfs {
    f_type: c_long,       /* type of file system (see below) */
    f_bsize: c_long,      /* optimal transfer block size */
    f_blocks: fsblkcnt_t, /* total data blocks in file system */
    f_bfree: fsblkcnt_t,  /* free blocks in fs */
    f_bavail: fsblkcnt_t, /* free blocks available to unprivileged user */
    f_files: fsfilcnt_t,  /* total file nodes in file system */
    f_ffree: fsfilcnt_t,  /* free file nodes in fs */
    f_fsid: c_long,       /* file system id */
    f_namelen: c_long,    /* maximum length of filenames */
    f_frsize: c_long,     /* fragment size (since Linux 2.6) */
    f_flags: c_long,
    f_spare: [c_long; 4],
}

// TODO
const ERRNO_MAX: usize = 4095;

pub fn e_raw(sys: usize) -> Result<usize, usize> {
    if sys > ERRNO_MAX.wrapping_neg() {
        Err(sys.wrapping_neg())
    } else {
        Ok(sys)
    }
}
pub fn e(sys: usize) -> usize {
    match e_raw(sys) {
        Ok(value) => value,
        Err(errcode) => {
            unsafe {
                errno = errcode as c_int;
            }
            !0
        }
    }
}

pub struct Sys;

impl Sys {
    // fn getrusage(who: c_int, r_usage: *mut rusage) -> c_int {
    //     e(unsafe { syscall!(GETRUSAGE, who, r_usage) }) as c_int
    // }

    pub unsafe fn ioctl(fd: c_int, request: c_ulong, out: *mut c_void) -> c_int {
        // TODO: Somehow support varargs to syscall??
        e(syscall!(IOCTL, fd, request, out)) as c_int
    }

    // fn times(out: *mut tms) -> clock_t {
    //     unsafe { syscall!(TIMES, out) as clock_t }
    // }
}

impl Pal for Sys {
    fn access(path: CStr, mode: c_int) -> c_int {
        e(unsafe { syscall!(ACCESS, path.as_ptr(), mode) }) as c_int
    }

    fn brk(addr: *mut c_void) -> *mut c_void {
        unsafe { syscall!(BRK, addr) as *mut c_void }
    }

    fn chdir(path: CStr) -> c_int {
        e(unsafe { syscall!(CHDIR, path.as_ptr()) }) as c_int
    }

    fn chmod(path: CStr, mode: mode_t) -> c_int {
        e(unsafe { syscall!(FCHMODAT, AT_FDCWD, path.as_ptr(), mode, 0) }) as c_int
    }

    fn chown(path: CStr, owner: uid_t, group: gid_t) -> c_int {
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

    fn clock_getres(clk_id: clockid_t, tp: *mut timespec) -> c_int {
        e(unsafe { syscall!(CLOCK_GETRES, clk_id, tp) }) as c_int
    }

    fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> c_int {
        e(unsafe { syscall!(CLOCK_GETTIME, clk_id, tp) }) as c_int
    }

    fn clock_settime(clk_id: clockid_t, tp: *const timespec) -> c_int {
        e(unsafe { syscall!(CLOCK_SETTIME, clk_id, tp) }) as c_int
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

    unsafe fn execve(path: CStr, argv: *const *mut c_char, envp: *const *mut c_char) -> c_int {
        e(syscall!(EXECVE, path.as_ptr(), argv, envp)) as c_int
    }
    unsafe fn fexecve(fildes: c_int, argv: *const *mut c_char, envp: *const *mut c_char) -> c_int {
        todo!("not yet used by relibc")
    }

    fn exit(status: c_int) -> ! {
        unsafe {
            syscall!(EXIT, status);
        }
        loop {}
    }
    fn exit_thread() -> ! {
        // TODO
        Self::exit(0)
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

    fn fdatasync(fildes: c_int) -> c_int {
        e(unsafe { syscall!(FDATASYNC, fildes) }) as c_int
    }

    fn flock(fd: c_int, operation: c_int) -> c_int {
        e(unsafe { syscall!(FLOCK, fd, operation) }) as c_int
    }

    fn fstat(fildes: c_int, buf: *mut stat) -> c_int {
        let empty = b"\0";
        let empty_ptr = empty.as_ptr() as *const c_char;
        e(unsafe { syscall!(NEWFSTATAT, fildes, empty_ptr, buf, AT_EMPTY_PATH) }) as c_int
    }

    fn fstatvfs(fildes: c_int, buf: *mut statvfs) -> c_int {
        let mut kbuf = linux_statfs::default();
        let kbuf_ptr = &mut kbuf as *mut linux_statfs;
        let res = e(unsafe { syscall!(FSTATFS, fildes, kbuf_ptr) }) as c_int;
        if res == 0 {
            unsafe {
                if !buf.is_null() {
                    (*buf).f_bsize = kbuf.f_bsize as c_ulong;
                    (*buf).f_frsize = if kbuf.f_frsize != 0 {
                        kbuf.f_frsize
                    } else {
                        kbuf.f_bsize
                    } as c_ulong;
                    (*buf).f_blocks = kbuf.f_blocks;
                    (*buf).f_bfree = kbuf.f_bfree;
                    (*buf).f_bavail = kbuf.f_bavail;
                    (*buf).f_files = kbuf.f_files;
                    (*buf).f_ffree = kbuf.f_ffree;
                    (*buf).f_favail = kbuf.f_ffree;
                    (*buf).f_fsid = kbuf.f_fsid as c_ulong;
                    (*buf).f_flag = kbuf.f_flags as c_ulong;
                    (*buf).f_namemax = kbuf.f_namelen as c_ulong;
                }
            }
        }
        res
    }

    fn fcntl(fildes: c_int, cmd: c_int, arg: c_ulonglong) -> c_int {
        e(unsafe { syscall!(FCNTL, fildes, cmd, arg) }) as c_int
    }

    fn fork() -> pid_t {
        e(unsafe { syscall!(CLONE, SIGCHLD, 0, 0, 0, 0) }) as pid_t
    }

    fn fpath(fildes: c_int, out: &mut [u8]) -> ssize_t {
        let mut proc_path = b"/proc/self/fd/".to_vec();
        write!(proc_path, "{}", fildes).unwrap();
        proc_path.push(0);

        Self::readlink(CStr::from_bytes_with_nul(&proc_path).unwrap(), out)
    }

    fn fsync(fildes: c_int) -> c_int {
        e(unsafe { syscall!(FSYNC, fildes) }) as c_int
    }

    fn ftruncate(fildes: c_int, length: off_t) -> c_int {
        e(unsafe { syscall!(FTRUNCATE, fildes, length) }) as c_int
    }

    fn futex(
        addr: *mut c_int,
        op: c_int,
        val: c_int,
        val2: usize,
    ) -> Result<c_long, crate::pthread::Errno> {
        e_raw(unsafe { syscall!(FUTEX, addr, op, val, val2, 0, 0) })
            .map(|r| r as c_long)
            .map_err(|e| crate::pthread::Errno(e as c_int))
    }

    fn futimens(fd: c_int, times: *const timespec) -> c_int {
        e(unsafe { syscall!(UTIMENSAT, fd, ptr::null::<c_char>(), times, 0) }) as c_int
    }

    fn utimens(path: CStr, times: *const timespec) -> c_int {
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

    unsafe fn getgroups(size: c_int, list: *mut gid_t) -> c_int {
        e(unsafe { syscall!(GETGROUPS, size, list) }) as c_int
    }

    fn getpagesize() -> usize {
        4096
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

    fn getpriority(which: c_int, who: id_t) -> c_int {
        e(unsafe { syscall!(GETPRIORITY, which, who) }) as c_int
    }

    fn getrandom(buf: &mut [u8], flags: c_uint) -> ssize_t {
        e(unsafe { syscall!(GETRANDOM, buf.as_mut_ptr(), buf.len(), flags) }) as ssize_t
    }

    unsafe fn getrlimit(resource: c_int, rlim: *mut rlimit) -> c_int {
        e(syscall!(GETRLIMIT, resource, rlim)) as c_int
    }

    unsafe fn setrlimit(resource: c_int, rlimit: *const rlimit) -> c_int {
        e(syscall!(SETRLIMIT, resource, rlimit)) as c_int
    }

    fn getsid(pid: pid_t) -> pid_t {
        e(unsafe { syscall!(GETSID, pid) }) as pid_t
    }

    fn gettid() -> pid_t {
        e(unsafe { syscall!(GETTID) }) as pid_t
    }

    fn gettimeofday(tp: *mut timeval, tzp: *mut timezone) -> c_int {
        e(unsafe { syscall!(GETTIMEOFDAY, tp, tzp) }) as c_int
    }

    fn getuid() -> uid_t {
        e(unsafe { syscall!(GETUID) }) as uid_t
    }

    fn lchown(path: CStr, owner: uid_t, group: gid_t) -> c_int {
        e(unsafe { syscall!(LCHOWN, path.as_ptr(), owner, group) }) as c_int
    }

    fn link(path1: CStr, path2: CStr) -> c_int {
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

    fn mkdir(path: CStr, mode: mode_t) -> c_int {
        e(unsafe { syscall!(MKDIRAT, AT_FDCWD, path.as_ptr(), mode) }) as c_int
    }

    fn mknodat(dir_fildes: c_int, path: CStr, mode: mode_t, dev: dev_t) -> c_int {
        // Note: dev_t is c_long (i64) and __kernel_dev_t is u32; So we need to cast it
        //       and check for overflow
        let k_dev: c_uint = dev as c_uint;
        if k_dev as dev_t != dev {
            return e(EINVAL as usize) as c_int;
        }

        e(unsafe { syscall!(MKNODAT, dir_fildes, path.as_ptr(), mode, k_dev) }) as c_int
    }

    fn mknod(path: CStr, mode: mode_t, dev: dev_t) -> c_int {
        Sys::mknodat(AT_FDCWD, path, mode, dev)
    }

    fn mkfifo(path: CStr, mode: mode_t) -> c_int {
        Sys::mknod(path, mode | S_IFIFO, 0)
    }

    unsafe fn mlock(addr: *const c_void, len: usize) -> c_int {
        e(syscall!(MLOCK, addr, len)) as c_int
    }

    fn mlockall(flags: c_int) -> c_int {
        e(unsafe { syscall!(MLOCKALL, flags) }) as c_int
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

    unsafe fn mremap(
        addr: *mut c_void,
        len: usize,
        new_len: usize,
        flags: c_int,
        args: *mut c_void,
    ) -> *mut c_void {
        e(syscall!(MREMAP, addr, len, new_len, flags, args)) as *mut c_void
    }

    unsafe fn mprotect(addr: *mut c_void, len: usize, prot: c_int) -> c_int {
        e(syscall!(MPROTECT, addr, len, prot)) as c_int
    }

    unsafe fn msync(addr: *mut c_void, len: usize, flags: c_int) -> c_int {
        e(syscall!(MSYNC, addr, len, flags)) as c_int
    }

    unsafe fn munlock(addr: *const c_void, len: usize) -> c_int {
        e(syscall!(MUNLOCK, addr, len)) as c_int
    }

    fn munlockall() -> c_int {
        e(unsafe { syscall!(MUNLOCKALL) }) as c_int
    }

    unsafe fn munmap(addr: *mut c_void, len: usize) -> c_int {
        e(syscall!(MUNMAP, addr, len)) as c_int
    }

    unsafe fn madvise(addr: *mut c_void, len: usize, flags: c_int) -> c_int {
        e(syscall!(MADVISE, addr, len, flags)) as c_int
    }

    fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> c_int {
        e(unsafe { syscall!(NANOSLEEP, rqtp, rmtp) }) as c_int
    }

    fn open(path: CStr, oflag: c_int, mode: mode_t) -> c_int {
        e(unsafe { syscall!(OPENAT, AT_FDCWD, path.as_ptr(), oflag, mode) }) as c_int
    }

    fn pipe2(fildes: &mut [c_int], flags: c_int) -> c_int {
        e(unsafe { syscall!(PIPE2, fildes.as_mut_ptr(), flags) }) as c_int
    }

    #[cfg(target_arch = "x86_64")]
    unsafe fn rlct_clone(
        stack: *mut usize,
    ) -> Result<crate::pthread::OsTid, crate::pthread::Errno> {
        let flags = CLONE_VM | CLONE_FS | CLONE_FILES | CLONE_SIGHAND | CLONE_THREAD;
        let pid;
        asm!("
            # Call clone syscall
            syscall

            # Check if child or parent
            test rax, rax
            jnz 2f

            # Load registers
            pop rax
            pop rdi
            pop rsi
            pop rdx
            pop rcx
            pop r8
            pop r9

            # Call entry point
            call rax

            # Exit
            mov rax, 60
            xor rdi, rdi
            syscall

            # Invalid instruction on failure to exit
            ud2

            # Return PID if parent
            2:
            ",
            inout("rax") SYS_CLONE => pid,
            inout("rdi") flags => _,
            inout("rsi") stack => _,
            inout("rdx") 0 => _,
            inout("r10") 0 => _,
            inout("r8") 0 => _,
            //TODO: out("rbx") _,
            out("rcx") _,
            out("r9") _,
            out("r11") _,
            out("r12") _,
            out("r13") _,
            out("r14") _,
            out("r15") _,
        );
        let tid = e_raw(pid).map_err(|err| crate::pthread::Errno(err as c_int))?;

        Ok(crate::pthread::OsTid { thread_id: tid })
    }
    unsafe fn rlct_kill(
        os_tid: crate::pthread::OsTid,
        signal: usize,
    ) -> Result<(), crate::pthread::Errno> {
        let tgid = Self::getpid();
        e_raw(unsafe { syscall!(TGKILL, tgid, os_tid.thread_id, signal) })
            .map(|_| ())
            .map_err(|err| crate::pthread::Errno(err as c_int))
    }
    fn current_os_tid() -> crate::pthread::OsTid {
        crate::pthread::OsTid {
            thread_id: unsafe { syscall!(GETTID) },
        }
    }

    fn read(fildes: c_int, buf: &mut [u8]) -> ssize_t {
        e(unsafe { syscall!(READ, fildes, buf.as_mut_ptr(), buf.len()) }) as ssize_t
    }

    fn readlink(pathname: CStr, out: &mut [u8]) -> ssize_t {
        e(unsafe {
            syscall!(
                READLINKAT,
                AT_FDCWD,
                pathname.as_ptr(),
                out.as_mut_ptr(),
                out.len()
            )
        }) as ssize_t
    }

    fn rename(old: CStr, new: CStr) -> c_int {
        e(unsafe { syscall!(RENAMEAT, AT_FDCWD, old.as_ptr(), AT_FDCWD, new.as_ptr()) }) as c_int
    }

    fn rmdir(path: CStr) -> c_int {
        e(unsafe { syscall!(UNLINKAT, AT_FDCWD, path.as_ptr(), AT_REMOVEDIR) }) as c_int
    }

    fn sched_yield() -> c_int {
        e(unsafe { syscall!(SCHED_YIELD) }) as c_int
    }

    unsafe fn setgroups(size: size_t, list: *const gid_t) -> c_int {
        e(unsafe { syscall!(SETGROUPS, size, list) }) as c_int
    }

    fn setpgid(pid: pid_t, pgid: pid_t) -> c_int {
        e(unsafe { syscall!(SETPGID, pid, pgid) }) as c_int
    }

    fn setpriority(which: c_int, who: id_t, prio: c_int) -> c_int {
        e(unsafe { syscall!(SETPRIORITY, which, who, prio) }) as c_int
    }

    fn setregid(rgid: gid_t, egid: gid_t) -> c_int {
        e(unsafe { syscall!(SETREGID, rgid, egid) }) as c_int
    }

    fn setreuid(ruid: uid_t, euid: uid_t) -> c_int {
        e(unsafe { syscall!(SETREUID, ruid, euid) }) as c_int
    }

    fn setsid() -> c_int {
        e(unsafe { syscall!(SETSID) }) as c_int
    }

    fn symlink(path1: CStr, path2: CStr) -> c_int {
        e(unsafe { syscall!(SYMLINKAT, path1.as_ptr(), AT_FDCWD, path2.as_ptr()) }) as c_int
    }

    fn sync() -> c_int {
        e(unsafe { syscall!(SYNC) }) as c_int
    }

    fn umask(mask: mode_t) -> mode_t {
        unsafe { syscall!(UMASK, mask) as mode_t }
    }

    fn uname(utsname: *mut utsname) -> c_int {
        e(unsafe { syscall!(UNAME, utsname, 0) }) as c_int
    }

    fn unlink(path: CStr) -> c_int {
        e(unsafe { syscall!(UNLINKAT, AT_FDCWD, path.as_ptr(), 0) }) as c_int
    }

    fn waitpid(pid: pid_t, stat_loc: *mut c_int, options: c_int) -> pid_t {
        e(unsafe { syscall!(WAIT4, pid, stat_loc, options, 0) }) as pid_t
    }

    fn write(fildes: c_int, buf: &[u8]) -> ssize_t {
        e(unsafe { syscall!(WRITE, fildes, buf.as_ptr(), buf.len()) }) as ssize_t
    }

    fn verify() -> bool {
        // GETPID on Linux is 39, which does not exist on Redox
        e(unsafe { sc::syscall5(sc::nr::GETPID, !0, !0, !0, !0, !0) }) != !0
    }
}
