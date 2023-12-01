use core::{arch::asm, ptr};
use core_io::Write;

use super::{errno, types::*, Pal};
use crate::{
    c_str::CStr,
    errno::Errno,
    header::{dirent::dirent, signal::SIGCHLD, sys_stat::S_IFIFO},
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

pub fn e_raw(sys: usize) -> Result<usize, Errno> {
    if sys > ERRNO_MAX.wrapping_neg() {
        Err(Errno(sys.wrapping_neg() as c_int))
    } else {
        Ok(sys)
    }
}

pub struct Sys;

impl Sys {
    // fn getrusage(who: c_int, r_usage: *mut rusage) -> Result<(), Errno> {
    //     e_raw(unsafe { syscall!(GETRUSAGE, who, r_usage) })?;
    // Ok(())
    // }

    pub unsafe fn ioctl(fd: c_int, request: c_ulong, out: *mut c_void) -> Result<c_int, Errno> {
        // TODO: Somehow support varargs to syscall??
        e_raw(syscall!(IOCTL, fd, request, out)).map(|res| res as c_int)
    }

    // fn times(out: *mut tms) -> clock_t {
    //     unsafe { syscall!(TIMES, out) as clock_t }
    // }
}

impl Pal for Sys {
    fn access(path: CStr, mode: c_int) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(ACCESS, path.as_ptr(), mode) })?;
        Ok(())
    }

    fn brk(addr: *mut c_void) -> Result<*mut c_void, Errno> {
        e_raw(unsafe { syscall!(BRK, addr) }).map(|res| res as *mut c_void)
    }

    fn chdir(path: CStr) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(CHDIR, path.as_ptr()) })?;
        Ok(())
    }

    fn chmod(path: CStr, mode: mode_t) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(FCHMODAT, AT_FDCWD, path.as_ptr(), mode, 0) })?;
        Ok(())
    }

    fn chown(path: CStr, owner: uid_t, group: gid_t) -> Result<(), Errno> {
        e_raw(unsafe {
            syscall!(
                FCHOWNAT,
                AT_FDCWD,
                path.as_ptr(),
                owner as u32,
                group as u32
            )
        })?;
        Ok(())
    }

    fn clock_getres(clk_id: clockid_t, tp: *mut timespec) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(CLOCK_GETRES, clk_id, tp) })?;
        Ok(())
    }

    fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(CLOCK_GETTIME, clk_id, tp) })?;
        Ok(())
    }

    fn clock_settime(clk_id: clockid_t, tp: *const timespec) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(CLOCK_SETTIME, clk_id, tp) })?;
        Ok(())
    }

    fn close(fildes: c_int) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(CLOSE, fildes) })?;
        Ok(())
    }

    fn dup(fildes: c_int) -> Result<c_int, Errno> {
        e_raw(unsafe { syscall!(DUP, fildes) }).map(|res| res as c_int)
    }

    fn dup2(fildes: c_int, fildes2: c_int) -> Result<c_int, Errno> {
        e_raw(unsafe { syscall!(DUP3, fildes, fildes2, 0) }).map(|res| res as c_int)
    }

    unsafe fn execve(
        path: CStr,
        argv: *const *mut c_char,
        envp: *const *mut c_char,
    ) -> Result<(), Errno> {
        e_raw(syscall!(EXECVE, path.as_ptr(), argv, envp))?;
        Ok(())
    }
    unsafe fn fexecve(
        fildes: c_int,
        argv: *const *mut c_char,
        envp: *const *mut c_char,
    ) -> Result<(), Errno> {
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

    fn fchdir(fildes: c_int) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(FCHDIR, fildes) })?;
        Ok(())
    }

    fn fchmod(fildes: c_int, mode: mode_t) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(FCHMOD, fildes, mode) })?;
        Ok(())
    }

    fn fchown(fildes: c_int, owner: uid_t, group: gid_t) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(FCHOWN, fildes, owner, group) })?;
        Ok(())
    }

    fn fdatasync(fildes: c_int) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(FDATASYNC, fildes) })?;
        Ok(())
    }

    fn flock(fd: c_int, operation: c_int) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(FLOCK, fd, operation) })?;
        Ok(())
    }

    fn fstat(fildes: c_int, buf: *mut stat) -> Result<(), Errno> {
        let empty = b"\0";
        let empty_ptr = empty.as_ptr() as *const c_char;
        e_raw(unsafe { syscall!(NEWFSTATAT, fildes, empty_ptr, buf, AT_EMPTY_PATH) })?;
        Ok(())
    }

    fn fstatvfs(fildes: c_int, buf: *mut statvfs) -> Result<(), Errno> {
        let mut kbuf = linux_statfs::default();
        let kbuf_ptr = &mut kbuf as *mut linux_statfs;
        e_raw(unsafe { syscall!(FSTATFS, fildes, kbuf_ptr) }).map(|res| unsafe {
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
        })
    }

    fn fcntl(fildes: c_int, cmd: c_int, arg: c_ulonglong) -> Result<c_int, Errno> {
        e_raw(unsafe { syscall!(FCNTL, fildes, cmd, arg) }).map(|res| res as c_int)
    }

    fn fork() -> Result<pid_t, Errno> {
        e_raw(unsafe { syscall!(CLONE, SIGCHLD, 0, 0, 0, 0) }).map(|res| res as pid_t)
    }

    fn fpath(fildes: c_int, out: &mut [u8]) -> Result<ssize_t, Errno> {
        let mut proc_path = b"/proc/self/fd/".to_vec();
        write!(proc_path, "{}", fildes).unwrap();
        proc_path.push(0);

        Self::readlink(CStr::from_bytes_with_nul(&proc_path).unwrap(), out)
    }

    fn fsync(fildes: c_int) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(FSYNC, fildes) })?;
        Ok(())
    }

    fn ftruncate(fildes: c_int, length: off_t) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(FTRUNCATE, fildes, length) })?;
        Ok(())
    }

    fn futex(
        addr: *mut c_int,
        op: c_int,
        val: c_int,
        val2: usize,
    ) -> Result<c_long, crate::errno::Errno> {
        e_raw(unsafe { syscall!(FUTEX, addr, op, val, val2, 0, 0) }).map(|r| r as c_long)
    }

    fn futimens(fd: c_int, times: *const timespec) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(UTIMENSAT, fd, ptr::null::<c_char>(), times, 0) })?;
        Ok(())
    }

    fn utimens(path: CStr, times: *const timespec) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(UTIMENSAT, AT_FDCWD, path.as_ptr(), times, 0) })?;
        Ok(())
    }

    fn getcwd(buf: *mut c_char, size: size_t) -> Result<*mut c_char, Errno> {
        e_raw(unsafe { syscall!(GETCWD, buf, size) }).map(|_| buf)
    }

    fn getdents(fd: c_int, dirents: *mut dirent, bytes: usize) -> Result<c_int, Errno> {
        e_raw(unsafe { syscall!(GETDENTS64, fd, dirents, bytes) }).map(|res| res as c_int)
    }

    fn getegid() -> gid_t {
        unsafe { syscall!(GETEGID) as gid_t }
    }

    fn geteuid() -> uid_t {
        unsafe { syscall!(GETEUID) as uid_t }
    }

    fn getgid() -> gid_t {
        unsafe { syscall!(GETGID) as gid_t }
    }

    unsafe fn getgroups(size: c_int, list: *mut gid_t) -> Result<c_int, Errno> {
        e_raw(unsafe { syscall!(GETGROUPS, size, list) }).map(|res| res as c_int)
    }

    fn getpagesize() -> usize {
        4096
    }

    fn getpgid(pid: pid_t) -> Result<pid_t, Errno> {
        e_raw(unsafe { syscall!(GETPGID, pid) }).map(|res| res as pid_t)
    }

    fn getpid() -> pid_t {
        unsafe { syscall!(GETPID) as pid_t }
    }

    fn getppid() -> pid_t {
        unsafe { syscall!(GETPPID) as pid_t }
    }

    fn getpriority(which: c_int, who: id_t) -> Result<c_int, Errno> {
        e_raw(unsafe { syscall!(GETPRIORITY, which, who) }).map(|res| res as c_int)
    }

    fn getrandom(buf: &mut [u8], flags: c_uint) -> Result<ssize_t, Errno> {
        e_raw(unsafe { syscall!(GETRANDOM, buf.as_mut_ptr(), buf.len(), flags) })
            .map(|res| res as ssize_t)
    }

    unsafe fn getrlimit(resource: c_int, rlim: *mut rlimit) -> Result<(), Errno> {
        e_raw(syscall!(GETRLIMIT, resource, rlim))?;
        Ok(())
    }

    unsafe fn setrlimit(resource: c_int, rlimit: *const rlimit) -> Result<(), Errno> {
        e_raw(syscall!(SETRLIMIT, resource, rlimit))?;
        Ok(())
    }

    fn getsid(pid: pid_t) -> Result<pid_t, Errno> {
        e_raw(unsafe { syscall!(GETSID, pid) }).map(|res| res as pid_t)
    }

    fn gettid() -> pid_t {
        unsafe { syscall!(GETTID) as pid_t }
    }

    fn gettimeofday(tp: *mut timeval, tzp: *mut timezone) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(GETTIMEOFDAY, tp, tzp) })?;
        Ok(())
    }

    fn getuid() -> uid_t {
        unsafe { syscall!(GETUID) as uid_t }
    }

    fn lchown(path: CStr, owner: uid_t, group: gid_t) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(LCHOWN, path.as_ptr(), owner, group) })?;
        Ok(())
    }

    fn link(path1: CStr, path2: CStr) -> Result<(), Errno> {
        e_raw(unsafe {
            syscall!(
                LINKAT,
                AT_FDCWD,
                path1.as_ptr(),
                AT_FDCWD,
                path2.as_ptr(),
                0
            )
        })?;
        Ok(())
    }

    fn lseek(fildes: c_int, offset: off_t, whence: c_int) -> Result<off_t, Errno> {
        e_raw(unsafe { syscall!(LSEEK, fildes, offset, whence) }).map(|res| res as off_t)
    }

    fn mkdir(path: CStr, mode: mode_t) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(MKDIRAT, AT_FDCWD, path.as_ptr(), mode) })?;
        Ok(())
    }

    fn mkfifo(path: CStr, mode: mode_t) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(MKNODAT, AT_FDCWD, path.as_ptr(), mode | S_IFIFO, 0) })?;
        Ok(())
    }

    unsafe fn mlock(addr: *const c_void, len: usize) -> Result<(), Errno> {
        e_raw(syscall!(MLOCK, addr, len))?;
        Ok(())
    }

    fn mlockall(flags: c_int) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(MLOCKALL, flags) })?;
        Ok(())
    }

    unsafe fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: c_int,
        flags: c_int,
        fildes: c_int,
        off: off_t,
    ) -> Result<*mut c_void, Errno> {
        e_raw(syscall!(MMAP, addr, len, prot, flags, fildes, off)).map(|res| res as *mut c_void)
    }

    unsafe fn mprotect(addr: *mut c_void, len: usize, prot: c_int) -> Result<(), Errno> {
        e_raw(syscall!(MPROTECT, addr, len, prot))?;
        Ok(())
    }

    unsafe fn msync(addr: *mut c_void, len: usize, flags: c_int) -> Result<(), Errno> {
        e_raw(syscall!(MSYNC, addr, len, flags))?;
        Ok(())
    }

    unsafe fn munlock(addr: *const c_void, len: usize) -> Result<(), Errno> {
        e_raw(syscall!(MUNLOCK, addr, len))?;
        Ok(())
    }

    fn munlockall() -> Result<(), Errno> {
        e_raw(unsafe { syscall!(MUNLOCKALL) })?;
        Ok(())
    }

    unsafe fn munmap(addr: *mut c_void, len: usize) -> Result<(), Errno> {
        e_raw(syscall!(MUNMAP, addr, len))?;
        Ok(())
    }

    unsafe fn madvise(addr: *mut c_void, len: usize, flags: c_int) -> Result<(), Errno> {
        e_raw(syscall!(MADVISE, addr, len, flags))?;
        Ok(())
    }

    fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(NANOSLEEP, rqtp, rmtp) })?;
        Ok(())
    }

    fn open(path: CStr, oflag: c_int, mode: mode_t) -> Result<c_int, Errno> {
        e_raw(unsafe { syscall!(OPENAT, AT_FDCWD, path.as_ptr(), oflag, mode) })
            .map(|res| res as c_int)
    }

    fn pipe2(fildes: &mut [c_int], flags: c_int) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(PIPE2, fildes.as_mut_ptr(), flags) })?;
        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    unsafe fn rlct_clone(stack: *mut usize) -> Result<crate::pthread::OsTid, crate::errno::Errno> {
        let flags = CLONE_VM | CLONE_FS | CLONE_FILES | CLONE_SIGHAND | CLONE_THREAD;
        let pid;
        asm!("
            # Call clone syscall
            syscall

            # Check if child or parent
            test rax, rax
            jnz 1f

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
            1:
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
        let tid = e_raw(pid)?;

        Ok(crate::pthread::OsTid { thread_id: tid })
    }
    unsafe fn rlct_kill(
        os_tid: crate::pthread::OsTid,
        signal: usize,
    ) -> Result<(), crate::errno::Errno> {
        let tgid = Self::getpid();
        e_raw(unsafe { syscall!(TGKILL, tgid, os_tid.thread_id, signal) })?;
        Ok(())
    }
    fn current_os_tid() -> crate::pthread::OsTid {
        crate::pthread::OsTid {
            thread_id: unsafe { syscall!(GETTID) },
        }
    }

    fn read(fildes: c_int, buf: &mut [u8]) -> Result<ssize_t, Errno> {
        e_raw(unsafe { syscall!(READ, fildes, buf.as_mut_ptr(), buf.len()) })
            .map(|res| res as ssize_t)
    }

    fn readlink(pathname: CStr, out: &mut [u8]) -> Result<ssize_t, Errno> {
        e_raw(unsafe {
            syscall!(
                READLINKAT,
                AT_FDCWD,
                pathname.as_ptr(),
                out.as_mut_ptr(),
                out.len()
            )
        })
        .map(|res| res as ssize_t)
    }

    fn rename(old: CStr, new: CStr) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(RENAMEAT, AT_FDCWD, old.as_ptr(), AT_FDCWD, new.as_ptr()) })?;
        Ok(())
    }

    fn rmdir(path: CStr) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(UNLINKAT, AT_FDCWD, path.as_ptr(), AT_REMOVEDIR) })?;
        Ok(())
    }

    fn sched_yield() -> Result<(), Errno> {
        e_raw(unsafe { syscall!(SCHED_YIELD) })?;
        Ok(())
    }

    unsafe fn setgroups(size: size_t, list: *const gid_t) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(SETGROUPS, size, list) })?;
        Ok(())
    }

    fn setpgid(pid: pid_t, pgid: pid_t) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(SETPGID, pid, pgid) })?;
        Ok(())
    }

    fn setpriority(which: c_int, who: id_t, prio: c_int) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(SETPRIORITY, which, who, prio) })?;
        Ok(())
    }

    fn setregid(rgid: gid_t, egid: gid_t) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(SETREGID, rgid, egid) })?;
        Ok(())
    }

    fn setreuid(ruid: uid_t, euid: uid_t) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(SETREUID, ruid, euid) })?;
        Ok(())
    }

    fn setsid() -> Result<(), Errno> {
        e_raw(unsafe { syscall!(SETSID) })?;
        Ok(())
    }

    fn symlink(path1: CStr, path2: CStr) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(SYMLINKAT, path1.as_ptr(), AT_FDCWD, path2.as_ptr()) })?;
        Ok(())
    }

    fn sync() -> Result<(), Errno> {
        e_raw(unsafe { syscall!(SYNC) })?;
        Ok(())
    }

    fn umask(mask: mode_t) -> mode_t {
        unsafe { syscall!(UMASK, mask) as mode_t }
    }

    fn uname(utsname: *mut utsname) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(UNAME, utsname, 0) })?;
        Ok(())
    }

    fn unlink(path: CStr) -> Result<(), Errno> {
        e_raw(unsafe { syscall!(UNLINKAT, AT_FDCWD, path.as_ptr(), 0) })?;
        Ok(())
    }

    fn waitpid(pid: pid_t, stat_loc: *mut c_int, options: c_int) -> Result<pid_t, Errno> {
        e_raw(unsafe { syscall!(WAIT4, pid, stat_loc, options, 0) }).map(|res| res as pid_t)
    }

    fn write(fildes: c_int, buf: &[u8]) -> Result<ssize_t, Errno> {
        e_raw(unsafe { syscall!(WRITE, fildes, buf.as_ptr(), buf.len()) }).map(|res| res as ssize_t)
    }

    fn verify() -> Result<(), Errno> {
        // GETPID on Linux is 39, which does not exist on Redox
        e_raw(unsafe { sc::syscall5(sc::nr::GETPID, !0, !0, !0, !0, !0) })?;
        Ok(())
    }
}
