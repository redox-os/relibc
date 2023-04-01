use core::{arch::asm, ptr};
use core_io::Write;

use super::{errno, types::*, Pal};
use crate::{
    c_str::CStr,
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

pub fn e(sys: usize) -> usize {
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
    // fn getrusage(who: c_int, r_usage: *mut rusage) -> c_int {
    //     e(unsafe { syscall!(GETRUSAGE, who, r_usage) }) as c_int
    // }

    pub unsafe fn ioctl(fd: c_int, request: c_ulong, out: *mut c_void) -> c_int {
        // TODO: Somehow support varargs to syscall??
        // e(syscall!(IOCTL, fd, request, out)) as c_int
        unimplemented!()
    }

    // fn times(out: *mut tms) -> clock_t {
    //     unsafe { syscall!(TIMES, out) as clock_t }
    // }
}

impl Pal for Sys {
    fn access(path: &CStr, mode: c_int) -> c_int {
        // e(unsafe { syscall!(ACCESS, path.as_ptr(), mode) }) as c_int
        unimplemented!()
    }

    fn brk(addr: *mut c_void) -> *mut c_void {
        unsafe { syscall!(SYS_BRK, addr) as *mut c_void }
    }

    fn chdir(path: &CStr) -> c_int {
        e(unsafe { syscall!(SYS_CHDIR, path.as_ptr()) }) as c_int
    }

    fn chmod(path: &CStr, mode: mode_t) -> c_int {
        // e(unsafe { syscall!(FCHMODAT, AT_FDCWD, path.as_ptr(), mode, 0) }) as c_int
        return 0;
    }

    fn chown(path: &CStr, owner: uid_t, group: gid_t) -> c_int {
        // e(unsafe {
        //     syscall!(
        //         FCHOWNAT,
        //         AT_FDCWD,
        //         path.as_ptr(),
        //         owner as u32,
        //         group as u32
        //     )
        // }) as c_int
        return 0;
    }

    fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> c_int {
        // e(unsafe { syscall!(CLOCK_GETTIME, clk_id, tp) }) as c_int
        unimplemented!()
    }

    fn close(fildes: c_int) -> c_int {
        e(unsafe { syscall!(SYS_CLOSE, fildes) }) as c_int
    }

    fn dup(fildes: c_int) -> c_int {
        // e(unsafe { syscall!(DUP, fildes) }) as c_int
        unimplemented!()
    }

    fn dup2(fildes: c_int, fildes2: c_int) -> c_int {
        // e(unsafe { syscall!(DUP3, fildes, fildes2, 0) }) as c_int
        unimplemented!()
    }

    unsafe fn execve(path: &CStr, argv: *const *mut c_char, envp: *const *mut c_char) -> c_int {
        e(syscall!(SYS_EXECVE, path.as_ptr(), argv, envp)) as c_int
    }

    fn exit(status: c_int) -> ! {
        unsafe {
            syscall!(SYS_EXIT, status);
        }
        loop {}
    }

    fn fchdir(fildes: c_int) -> c_int {
        // e(unsafe { syscall!(FCHDIR, fildes) }) as c_int
        unimplemented!()
    }

    fn fchmod(fildes: c_int, mode: mode_t) -> c_int {
        // e(unsafe { syscall!(FCHMOD, fildes, mode) }) as c_int
        return 0;
    }

    fn fchown(fildes: c_int, owner: uid_t, group: gid_t) -> c_int {
        // e(unsafe { syscall!(FCHOWN, fildes, owner, group) }) as c_int
        return 0;
    }

    fn flock(fd: c_int, operation: c_int) -> c_int {
        // e(unsafe { syscall!(FLOCK, fd, operation) }) as c_int
        return 0;
    }

    fn fstat(fildes: c_int, buf: *mut stat) -> c_int {
        // let empty = b"\0";
        // let empty_ptr = empty.as_ptr() as *const c_char;
        // e(unsafe { syscall!(NEWFSTATAT, fildes, empty_ptr, buf, AT_EMPTY_PATH) }) as c_int
        unimplemented!()
    }

    fn fstatvfs(fildes: c_int, buf: *mut statvfs) -> c_int {
        // let mut kbuf = linux_statfs::default();
        // let kbuf_ptr = &mut kbuf as *mut linux_statfs;
        // let res = e(unsafe { syscall!(FSTATFS, fildes, kbuf_ptr) }) as c_int;
        // if res == 0 {
        //     unsafe {
        //         if !buf.is_null() {
        //             (*buf).f_bsize = kbuf.f_bsize as c_ulong;
        //             (*buf).f_frsize = if kbuf.f_frsize != 0 {
        //                 kbuf.f_frsize
        //             } else {
        //                 kbuf.f_bsize
        //             } as c_ulong;
        //             (*buf).f_blocks = kbuf.f_blocks;
        //             (*buf).f_bfree = kbuf.f_bfree;
        //             (*buf).f_bavail = kbuf.f_bavail;
        //             (*buf).f_files = kbuf.f_files;
        //             (*buf).f_ffree = kbuf.f_ffree;
        //             (*buf).f_favail = kbuf.f_ffree;
        //             (*buf).f_fsid = kbuf.f_fsid as c_ulong;
        //             (*buf).f_flag = kbuf.f_flags as c_ulong;
        //             (*buf).f_namemax = kbuf.f_namelen as c_ulong;
        //         }
        //     }
        // }
        // res

        unimplemented!()
    }

    fn fcntl(fildes: c_int, cmd: c_int, arg: c_int) -> c_int {
        // e(unsafe { syscall!(FCNTL, fildes, cmd, arg) }) as c_int
        unimplemented!()
    }

    fn fork() -> pid_t {
        // e(unsafe { syscall!(CLONE, SIGCHLD, 0, 0, 0, 0) }) as pid_t
        e(unsafe { syscall!(SYS_FORK) }) as pid_t
    }

    fn fpath(fildes: c_int, out: &mut [u8]) -> ssize_t {
        // let mut proc_path = b"/proc/self/fd/".to_vec();
        // write!(proc_path, "{}", fildes).unwrap();
        // proc_path.push(0);

        // Self::readlink(CStr::from_bytes_with_nul(&proc_path).unwrap(), out)
        unimplemented!()
    }

    fn fsync(fildes: c_int) -> c_int {
        // e(unsafe { syscall!(FSYNC, fildes) }) as c_int
        return 0;
    }

    fn ftruncate(fildes: c_int, length: off_t) -> c_int {
        // e(unsafe { syscall!(FTRUNCATE, fildes, length) }) as c_int
        unimplemented!()
    }

    fn futex(addr: *mut c_int, op: c_int, val: c_int, val2: usize) -> c_int {
        // unsafe { syscall!(FUTEX, addr, op, val, val2, 0, 0) as c_int }
        unimplemented!()
    }

    fn futimens(fd: c_int, times: *const timespec) -> c_int {
        // e(unsafe { syscall!(UTIMENSAT, fd, ptr::null::<c_char>(), times, 0) }) as c_int
        unimplemented!()
    }

    fn utimens(path: &CStr, times: *const timespec) -> c_int {
        // e(unsafe { syscall!(UTIMENSAT, AT_FDCWD, path.as_ptr(), times, 0) }) as c_int
        unimplemented!()
    }

    fn getcwd(buf: *mut c_char, size: size_t) -> *mut c_char {
        // if e(unsafe { syscall!(GETCWD, buf, size) }) == !0 {
        //     ptr::null_mut()
        // } else {
        //     buf
        // }
        unimplemented!()
    }

    fn getdents(fd: c_int, dirents: *mut dirent, bytes: usize) -> c_int {
        unsafe { syscall!(SYS_GET_DENTS, fd, dirents, bytes) as c_int }
    }

    fn getegid() -> gid_t {
        // e(unsafe { syscall!(GETEGID) }) as gid_t
        return 0;
    }

    fn geteuid() -> uid_t {
        // e(unsafe { syscall!(GETEUID) }) as uid_t
        return 0;
    }

    fn getgid() -> gid_t {
        // e(unsafe { syscall!(GETGID) }) as gid_t
        return 0;
    }

    fn getpagesize() -> usize {
        // 4096
        1024 * 1024 * 2
    }

    fn getpgid(pid: pid_t) -> pid_t {
        // e(unsafe { syscall!(GETPGID, pid) }) as pid_t
        return 0;
    }

    fn getpid() -> pid_t {
        e(unsafe { syscall!(SYS_GETPID) }) as pid_t
    }

    fn getppid() -> pid_t {
        // e(unsafe { syscall!(GETPPID) }) as pid_t
        return 0;
    }

    fn getrandom(buf: &mut [u8], flags: c_uint) -> ssize_t {
        // e(unsafe { syscall!(GETRANDOM, buf.as_mut_ptr(), buf.len(), flags) }) as ssize_t
        unimplemented!()
    }

    unsafe fn getrlimit(resource: c_int, rlim: *mut rlimit) -> c_int {
        // e(syscall!(GETRLIMIT, resource, rlim)) as c_int
        unimplemented!()
    }

    fn getsid(pid: pid_t) -> pid_t {
        // e(unsafe { syscall!(GETSID, pid) }) as pid_t
        return 0;
    }

    fn gettid() -> pid_t {
        // e(unsafe { syscall!(GETTID) }) as pid_t
        return Self::getpid();
    }

    fn gettimeofday(tp: *mut timeval, tzp: *mut timezone) -> c_int {
        // e(unsafe { syscall!(GETTIMEOFDAY, tp, tzp) }) as c_int
        unimplemented!()
    }

    fn getuid() -> uid_t {
        // e(unsafe { syscall!(GETUID) }) as uid_t
        return 0;
    }

    fn lchown(path: &CStr, owner: uid_t, group: gid_t) -> c_int {
        // e(unsafe { syscall!(LCHOWN, path.as_ptr(), owner, group) }) as c_int
        return 0;
    }

    fn link(path1: &CStr, path2: &CStr) -> c_int {
        // e(unsafe {
        //     syscall!(
        //         LINKAT,
        //         AT_FDCWD,
        //         path1.as_ptr(),
        //         AT_FDCWD,
        //         path2.as_ptr(),
        //         0
        //     )
        // }) as c_int
        unimplemented!()
    }

    fn lseek(fildes: c_int, offset: off_t, whence: c_int) -> off_t {
        e(unsafe { syscall!(SYS_LSEEK, fildes, offset, whence) }) as off_t
    }

    fn mkdir(path: &CStr, mode: mode_t) -> c_int {
        // e(unsafe { syscall!(MKDIRAT, AT_FDCWD, path.as_ptr(), mode) }) as c_int
        e(unsafe { syscall!(SYS_MKDIR, path.as_ptr(), mode) }) as c_int
    }

    fn mkfifo(path: &CStr, mode: mode_t) -> c_int {
        // e(unsafe { syscall!(MKNODAT, AT_FDCWD, path.as_ptr(), mode | S_IFIFO, 0) }) as c_int
        unimplemented!()
    }

    unsafe fn mlock(addr: *const c_void, len: usize) -> c_int {
        // e(syscall!(MLOCK, addr, len)) as c_int
        unimplemented!()
    }

    fn mlockall(flags: c_int) -> c_int {
        // e(unsafe { syscall!(MLOCKALL, flags) }) as c_int
        unimplemented!()
    }

    unsafe fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: c_int,
        flags: c_int,
        fildes: c_int,
        off: off_t,
    ) -> *mut c_void {
        // e(syscall!(MMAP, addr, len, prot, flags, fildes, off)) as *mut c_void
        unimplemented!()
    }

    unsafe fn mprotect(addr: *mut c_void, len: usize, prot: c_int) -> c_int {
        // e(syscall!(MPROTECT, addr, len, prot)) as c_int
        unimplemented!()
    }

    unsafe fn msync(addr: *mut c_void, len: usize, flags: c_int) -> c_int {
        // e(syscall!(MSYNC, addr, len, flags)) as c_int
        unimplemented!()
    }

    unsafe fn munlock(addr: *const c_void, len: usize) -> c_int {
        // e(syscall!(MUNLOCK, addr, len)) as c_int
        unimplemented!()
    }

    fn munlockall() -> c_int {
        // e(unsafe { syscall!(MUNLOCKALL) }) as c_int
        unimplemented!()
    }

    unsafe fn munmap(addr: *mut c_void, len: usize) -> c_int {
        // e(syscall!(MUNMAP, addr, len)) as c_int
        unimplemented!()
    }

    fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> c_int {
        e(unsafe { syscall!(SYS_NANOSLEEP, rqtp, rmtp) }) as c_int
    }

    fn open(path: &CStr, oflag: c_int, mode: mode_t) -> c_int {
        e(unsafe { syscall!(SYS_OPEN, path.as_ptr(), oflag, mode) }) as c_int
    }

    fn pipe2(fildes: &mut [c_int], flags: c_int) -> c_int {
        // e(unsafe { syscall!(PIPE2, fildes.as_mut_ptr(), flags) }) as c_int
        unimplemented!()
    }

    #[cfg(target_arch = "x86_64")]
    unsafe fn pte_clone(stack: *mut usize) -> pid_t {
        unimplemented!()
        // let flags = CLONE_VM | CLONE_FS | CLONE_FILES | CLONE_SIGHAND | CLONE_THREAD;
        // let pid;
        // asm!("
        //     # Call clone syscall
        //     syscall

        //     # Check if child or parent
        //     test rax, rax
        //     jnz 1f

        //     # Load registers
        //     pop rax
        //     pop rdi
        //     pop rsi
        //     pop rdx
        //     pop rcx
        //     pop r8
        //     pop r9

        //     # Call entry point
        //     call rax

        //     # Exit
        //     mov rax, 60
        //     xor rdi, rdi
        //     syscall

        //     # Invalid instruction on failure to exit
        //     ud2

        //     # Return PID if parent
        //     1:
        //     ",
        //     inout("rax") SYS_CLONE => pid,
        //     inout("rdi") flags => _,
        //     inout("rsi") stack => _,
        //     inout("rdx") 0 => _,
        //     inout("r10") 0 => _,
        //     inout("r8") 0 => _,
        //     //TODO: out("rbx") _,
        //     out("rcx") _,
        //     out("r9") _,
        //     out("r11") _,
        //     out("r12") _,
        //     out("r13") _,
        //     out("r14") _,
        //     out("r15") _,
        // );
        // e(pid) as pid_t
    }

    fn read(fildes: c_int, buf: &mut [u8]) -> ssize_t {
        e(unsafe { syscall!(SYS_READ, fildes, buf.as_mut_ptr(), buf.len()) }) as ssize_t
    }

    fn readlink(pathname: &CStr, out: &mut [u8]) -> ssize_t {
        unimplemented!()
        // e(unsafe {
        //     syscall!(
        //         READLINKAT,
        //         AT_FDCWD,
        //         pathname.as_ptr(),
        //         out.as_mut_ptr(),
        //         out.len()
        //     )
        // }) as ssize_t
    }

    fn rename(old: &CStr, new: &CStr) -> c_int {
        // e(unsafe { syscall!(RENAMEAT, AT_FDCWD, old.as_ptr(), AT_FDCWD, new.as_ptr()) }) as c_int
        unimplemented!()
    }

    fn rmdir(path: &CStr) -> c_int {
        e(unsafe { syscall!(SYS_UNLINK_AT, 0, path.as_ptr(), AT_REMOVEDIR) }) as c_int
    }

    fn sched_yield() -> c_int {
        // e(unsafe { syscall!(SCHED_YIELD) }) as c_int
        unimplemented!()
    }

    fn setpgid(pid: pid_t, pgid: pid_t) -> c_int {
        // e(unsafe { syscall!(SETPGID, pid, pgid) }) as c_int
        unimplemented!()
    }

    fn setregid(rgid: gid_t, egid: gid_t) -> c_int {
        // e(unsafe { syscall!(SETREGID, rgid, egid) }) as c_int
        unimplemented!()
    }

    fn setreuid(ruid: uid_t, euid: uid_t) -> c_int {
        // e(unsafe { syscall!(SETREUID, ruid, euid) }) as c_int
        unimplemented!()
    }

    fn symlink(path1: &CStr, path2: &CStr) -> c_int {
        // e(unsafe { syscall!(SYMLINKAT, path1.as_ptr(), AT_FDCWD, path2.as_ptr()) }) as c_int
        unimplemented!()
    }

    fn umask(mask: mode_t) -> mode_t {
        // unsafe { syscall!(UMASK, mask) as mode_t }
        unimplemented!()
    }

    fn uname(utsname: *mut utsname) -> c_int {
        // e(unsafe { syscall!(UNAME, utsname, 0) }) as c_int
        unimplemented!()
    }

    fn unlink(path: &CStr) -> c_int {
        e(unsafe { syscall!(SYS_UNLINK_AT, AT_FDCWD, path.as_ptr(), 0) }) as c_int
    }

    fn waitpid(pid: pid_t, stat_loc: *mut c_int, options: c_int) -> pid_t {
        e(unsafe { syscall!(SYS_WAIT4, pid, stat_loc, options, 0) }) as pid_t
    }

    fn write(fildes: c_int, buf: &[u8]) -> ssize_t {
        e(unsafe { syscall!(SYS_WRITE, fildes, buf.as_ptr(), buf.len()) }) as ssize_t
    }

    fn verify() -> bool {
        // GETPID on Linux is 39, which does not exist on Redox
        // e(unsafe { dsc::syscall5(dsc::nr::GETPID, !0, !0, !0, !0, !0) }) != !0
        unimplemented!()
    }
}
