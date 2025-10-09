use core::{arch::asm, ptr};

use super::{ERRNO, Pal, types::*};
use crate::{
    c_str::CStr,
    error::{Errno, Result},
    header::{
        dirent::dirent,
        errno::{EINVAL, EIO, EOPNOTSUPP},
        fcntl::{AT_EMPTY_PATH, AT_FDCWD, AT_REMOVEDIR, AT_SYMLINK_NOFOLLOW},
        signal::{SIGCHLD, sigset_t},
        sys_resource::{rlimit, rusage},
        sys_select,
        sys_stat::{S_IFIFO, stat},
        sys_statvfs::statvfs,
        sys_time::{timeval, timezone},
        sys_utsname::utsname,
        time::timespec,
        unistd::{SEEK_CUR, SEEK_SET},
    },
    io::Write,
    out::Out,
};

mod epoll;
mod ptrace;
mod signal;
mod socket;

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

pub fn e_raw(sys: usize) -> Result<usize> {
    if sys > ERRNO_MAX.wrapping_neg() {
        Err(Errno(sys.wrapping_neg() as _))
    } else {
        Ok(sys)
    }
}

/// Linux syscall implementation of the platform abstraction layer.
pub struct Sys;

impl Sys {
    pub unsafe fn ioctl(fd: c_int, request: c_ulong, out: *mut c_void) -> Result<c_int> {
        // TODO: Somehow support varargs to syscall??
        Ok(e_raw(syscall!(IOCTL, fd, request, out))? as c_int)
    }

    // fn times(out: *mut tms) -> clock_t {
    //     unsafe { syscall!(TIMES, out) as clock_t }
    // }
}

impl Pal for Sys {
    fn access(path: CStr, mode: c_int) -> Result<()> {
        e_raw(unsafe { syscall!(ACCESS, path.as_ptr(), mode) }).map(|_| ())
    }

    unsafe fn brk(addr: *mut c_void) -> Result<*mut c_void> {
        Ok(e_raw(unsafe { syscall!(BRK, addr) })? as *mut c_void)
    }

    fn chdir(path: CStr) -> Result<()> {
        e_raw(unsafe { syscall!(CHDIR, path.as_ptr()) }).map(|_| ())
    }
    fn set_default_scheme(scheme: CStr) -> Result<()> {
        Err(Errno(EOPNOTSUPP))
    }

    fn chmod(path: CStr, mode: mode_t) -> Result<()> {
        e_raw(unsafe { syscall!(FCHMODAT, AT_FDCWD, path.as_ptr(), mode, 0) }).map(|_| ())
    }

    fn chown(path: CStr, owner: uid_t, group: gid_t) -> Result<()> {
        e_raw(unsafe {
            syscall!(
                FCHOWNAT,
                AT_FDCWD,
                path.as_ptr(),
                owner as u32,
                group as u32
            )
        })
        .map(|_| ())
    }

    fn clock_getres(clk_id: clockid_t, res: Option<Out<timespec>>) -> Result<()> {
        e_raw(unsafe {
            syscall!(
                CLOCK_GETRES,
                clk_id,
                res.map_or(core::ptr::null_mut(), |mut p| p.as_mut_ptr())
            )
        })
        .map(|_| ())
    }

    fn clock_gettime(clk_id: clockid_t, mut tp: Out<timespec>) -> Result<()> {
        e_raw(unsafe { syscall!(CLOCK_GETTIME, clk_id, tp.as_mut_ptr()) }).map(|_| ())
    }

    unsafe fn clock_settime(clk_id: clockid_t, tp: *const timespec) -> Result<()> {
        e_raw(syscall!(CLOCK_SETTIME, clk_id, tp)).map(|_| ())
    }

    fn close(fildes: c_int) -> Result<()> {
        e_raw(unsafe { syscall!(CLOSE, fildes) }).map(|_| ())
    }

    fn dup(fildes: c_int) -> Result<c_int> {
        e_raw(unsafe { syscall!(DUP, fildes) }).map(|f| f as c_int)
    }

    fn dup2(fildes: c_int, fildes2: c_int) -> Result<c_int> {
        e_raw(unsafe { syscall!(DUP3, fildes, fildes2, 0) }).map(|f| f as c_int)
    }

    unsafe fn execve(path: CStr, argv: *const *mut c_char, envp: *const *mut c_char) -> Result<()> {
        e_raw(syscall!(EXECVE, path.as_ptr(), argv, envp))?;
        unreachable!()
    }
    unsafe fn fexecve(
        fildes: c_int,
        argv: *const *mut c_char,
        envp: *const *mut c_char,
    ) -> Result<()> {
        todo!("not yet used by relibc")
    }

    fn exit(status: c_int) -> ! {
        unsafe {
            syscall!(EXIT, status);
        }
        loop {}
    }
    unsafe fn exit_thread(_stack_base: *mut (), _stack_size: usize) -> ! {
        // TODO
        Self::exit(0)
    }

    fn fchdir(fildes: c_int) -> Result<()> {
        e_raw(unsafe { syscall!(FCHDIR, fildes) }).map(|_| ())
    }

    fn fchmod(fildes: c_int, mode: mode_t) -> Result<()> {
        e_raw(unsafe { syscall!(FCHMOD, fildes, mode) }).map(|_| ())
    }

    fn fchmodat(dirfd: c_int, path: Option<CStr>, mode: mode_t, flags: c_int) -> Result<()> {
        e_raw(unsafe {
            syscall!(
                FCHMODAT,
                dirfd,
                path.map_or(core::ptr::null(), |p| p.as_ptr()),
                mode,
                flags
            )
        })
        .map(|_| ())
    }

    fn fchown(fildes: c_int, owner: uid_t, group: gid_t) -> Result<()> {
        e_raw(unsafe { syscall!(FCHOWN, fildes, owner, group) }).map(|_| ())
    }

    fn fdatasync(fildes: c_int) -> Result<()> {
        e_raw(unsafe { syscall!(FDATASYNC, fildes) }).map(|_| ())
    }

    fn flock(fd: c_int, operation: c_int) -> Result<()> {
        e_raw(unsafe { syscall!(FLOCK, fd, operation) }).map(|_| ())
    }

    fn fstat(fildes: c_int, mut buf: Out<stat>) -> Result<()> {
        let empty = b"\0";
        let empty_ptr = empty.as_ptr() as *const c_char;
        e_raw(unsafe {
            syscall!(
                NEWFSTATAT,
                fildes,
                empty_ptr,
                buf.as_mut_ptr(),
                AT_EMPTY_PATH
            )
        })
        .map(|_| ())
    }

    fn fstatat(fildes: c_int, path: Option<CStr>, mut buf: Out<stat>, flags: c_int) -> Result<()> {
        e_raw(unsafe {
            syscall!(
                NEWFSTATAT,
                fildes,
                path.map_or(core::ptr::null(), |s| s.as_ptr()),
                buf.as_mut_ptr(),
                flags
            )
        })
        .map(|_| ())
    }

    fn fstatvfs(fildes: c_int, mut buf: Out<statvfs>) -> Result<()> {
        let buf = buf.as_mut_ptr();

        let mut kbuf = linux_statfs::default();
        let kbuf_ptr = &mut kbuf as *mut linux_statfs;
        e_raw(unsafe { syscall!(FSTATFS, fildes, kbuf_ptr) })?;

        if !buf.is_null() {
            unsafe {
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
        Ok(())
    }

    fn fcntl(fildes: c_int, cmd: c_int, arg: c_ulonglong) -> Result<c_int> {
        Ok(e_raw(unsafe { syscall!(FCNTL, fildes, cmd, arg) })? as c_int)
    }

    unsafe fn fork() -> Result<pid_t> {
        Ok(e_raw(unsafe { syscall!(CLONE, SIGCHLD, 0, 0, 0, 0) })? as pid_t)
    }

    fn fpath(fildes: c_int, out: &mut [u8]) -> Result<usize> {
        let proc_path = format!("/proc/self/fd/{}\0", fildes).into_bytes();
        Self::readlink(CStr::from_bytes_with_nul(&proc_path).unwrap(), out)
    }

    fn fsync(fildes: c_int) -> Result<()> {
        e_raw(unsafe { syscall!(FSYNC, fildes) }).map(|_| ())
    }

    fn ftruncate(fildes: c_int, length: off_t) -> Result<()> {
        e_raw(unsafe { syscall!(FTRUNCATE, fildes, length) }).map(|_| ())
    }

    #[inline]
    unsafe fn futex_wait(addr: *mut u32, val: u32, deadline: Option<&timespec>) -> Result<()> {
        let deadline = deadline.map_or(0, |d| d as *const _ as usize);
        e_raw(unsafe {
            syscall!(
                FUTEX, addr,       // uaddr
                9,          // futex_op: FUTEX_WAIT_BITSET
                val,        // val
                deadline,   // timeout: deadline
                0,          // uaddr2/val2: 0/NULL
                0xffffffff  // val3: FUTEX_BITSET_MATCH_ANY
            )
        })
        .map(|_| ())
    }
    #[inline]
    unsafe fn futex_wake(addr: *mut u32, num: u32) -> Result<u32> {
        e_raw(unsafe {
            syscall!(FUTEX, addr, 1 /* FUTEX_WAKE */, num)
        })
        .map(|n| n as u32)
    }

    unsafe fn futimens(fd: c_int, times: *const timespec) -> Result<()> {
        e_raw(unsafe { syscall!(UTIMENSAT, fd, ptr::null::<c_char>(), times, 0) }).map(|_| ())
    }

    unsafe fn utimens(path: CStr, times: *const timespec) -> Result<()> {
        e_raw(unsafe { syscall!(UTIMENSAT, AT_FDCWD, path.as_ptr(), times, 0) }).map(|_| ())
    }

    fn getcwd(mut buf: Out<[u8]>) -> Result<()> {
        e_raw(unsafe {
            syscall!(
                GETCWD,
                buf.as_mut_ptr().as_mut_ptr(),
                buf.as_mut_ptr().len()
            )
        })?;
        Ok(())
    }

    fn getdents(fd: c_int, buf: &mut [u8], _off: u64) -> Result<usize> {
        e_raw(unsafe { syscall!(GETDENTS64, fd, buf.as_mut_ptr(), buf.len()) })
    }
    fn dir_seek(fd: c_int, off: u64) -> Result<()> {
        e_raw(unsafe { syscall!(LSEEK, fd, off, SEEK_SET) })?;
        Ok(())
    }
    unsafe fn dent_reclen_offset(this_dent: &[u8], offset: usize) -> Option<(u16, u64)> {
        let dent = this_dent.as_ptr().cast::<dirent>();
        Some(((*dent).d_reclen, (*dent).d_off as u64))
    }

    fn getegid() -> gid_t {
        // Always successful
        unsafe { syscall!(GETEGID) as gid_t }
    }

    fn geteuid() -> uid_t {
        // Always successful
        unsafe { syscall!(GETEUID) as uid_t }
    }

    fn getgid() -> gid_t {
        // Always successful
        unsafe { syscall!(GETGID) as gid_t }
    }

    fn getgroups(mut list: Out<[gid_t]>) -> Result<c_int> {
        Ok(e_raw(unsafe {
            syscall!(
                GETGROUPS,
                list.len() as c_int,
                list.as_mut_ptr().as_mut_ptr()
            )
        })? as c_int)
    }

    fn getpagesize() -> usize {
        4096
    }

    fn getpgid(pid: pid_t) -> Result<pid_t> {
        Ok(e_raw(unsafe { syscall!(GETPGID, pid) })? as pid_t)
    }

    fn getpid() -> pid_t {
        // Always successful
        unsafe { syscall!(GETPID) as pid_t }
    }

    fn getppid() -> pid_t {
        // Always successful
        unsafe { syscall!(GETPPID) as pid_t }
    }

    fn getpriority(which: c_int, who: id_t) -> Result<c_int> {
        Ok(e_raw(unsafe { syscall!(GETPRIORITY, which, who) })? as c_int)
    }

    fn getrandom(buf: &mut [u8], flags: c_uint) -> Result<usize> {
        e_raw(unsafe { syscall!(GETRANDOM, buf.as_mut_ptr(), buf.len(), flags) })
    }

    fn getrlimit(resource: c_int, mut rlim: Out<rlimit>) -> Result<()> {
        e_raw(unsafe { syscall!(GETRLIMIT, resource, rlim.as_mut_ptr()) }).map(|_| ())
    }

    fn getresgid(
        rgid: Option<Out<gid_t>>,
        egid: Option<Out<gid_t>>,
        sgid: Option<Out<gid_t>>,
    ) -> Result<()> {
        unsafe {
            e_raw(syscall!(
                GETRESGID,
                rgid.map_or(0, |mut r| r.as_mut_ptr() as usize),
                egid.map_or(0, |mut r| r.as_mut_ptr() as usize),
                sgid.map_or(0, |mut r| r.as_mut_ptr() as usize)
            ))
            .map(|_| ())
        }
    }
    fn getresuid(
        ruid: Option<Out<uid_t>>,
        euid: Option<Out<uid_t>>,
        suid: Option<Out<uid_t>>,
    ) -> Result<()> {
        unsafe {
            e_raw(syscall!(
                GETRESUID,
                ruid.map_or(0, |mut r| r.as_mut_ptr() as usize),
                euid.map_or(0, |mut r| r.as_mut_ptr() as usize),
                suid.map_or(0, |mut r| r.as_mut_ptr() as usize)
            ))
            .map(|_| ())
        }
    }

    unsafe fn setrlimit(resource: c_int, rlimit: *const rlimit) -> Result<()> {
        e_raw(syscall!(SETRLIMIT, resource, rlimit)).map(|_| ())
    }

    fn getrusage(who: c_int, mut r_usage: Out<rusage>) -> Result<()> {
        e_raw(unsafe { syscall!(GETRUSAGE, who, r_usage.as_mut_ptr()) })?;
        Ok(())
    }

    fn getsid(pid: pid_t) -> Result<pid_t> {
        Ok(e_raw(unsafe { syscall!(GETSID, pid) })? as pid_t)
    }

    fn gettid() -> pid_t {
        // Always successful
        unsafe { syscall!(GETTID) as pid_t }
    }

    fn gettimeofday(mut tp: Out<timeval>, tzp: Option<Out<timezone>>) -> Result<()> {
        e_raw(unsafe {
            syscall!(
                GETTIMEOFDAY,
                tp.as_mut_ptr(),
                tzp.map_or(0, |mut p| p.as_mut_ptr() as usize)
            )
        })
        .map(|_| ())
    }

    fn getuid() -> uid_t {
        unsafe { syscall!(GETUID) as uid_t }
    }

    fn lchown(path: CStr, owner: uid_t, group: gid_t) -> Result<()> {
        e_raw(unsafe { syscall!(LCHOWN, path.as_ptr(), owner, group) }).map(|_| ())
    }

    fn link(path1: CStr, path2: CStr) -> Result<()> {
        e_raw(unsafe {
            syscall!(
                LINKAT,
                AT_FDCWD,
                path1.as_ptr(),
                AT_FDCWD,
                path2.as_ptr(),
                0
            )
        })
        .map(|_| ())
    }

    fn lseek(fildes: c_int, offset: off_t, whence: c_int) -> Result<off_t> {
        e_raw(unsafe { syscall!(LSEEK, fildes, offset, whence) }).map(|o| o as off_t)
    }

    fn mkdir(path: CStr, mode: mode_t) -> Result<()> {
        e_raw(unsafe { syscall!(MKDIRAT, AT_FDCWD, path.as_ptr(), mode) }).map(|_| ())
    }

    fn mknodat(dir_fildes: c_int, path: CStr, mode: mode_t, dev: dev_t) -> Result<()> {
        // Note: dev_t is c_long (i64) and __kernel_dev_t is u32; So we need to cast it
        //       and check for overflow
        let k_dev: c_uint = dev as c_uint;
        if k_dev as dev_t != dev {
            return Err(Errno(EINVAL));
        }

        e_raw(unsafe { syscall!(MKNODAT, dir_fildes, path.as_ptr(), mode, k_dev) }).map(|_| ())
    }

    fn mknod(path: CStr, mode: mode_t, dev: dev_t) -> Result<()> {
        Sys::mknodat(AT_FDCWD, path, mode, dev)
    }

    fn mkfifo(path: CStr, mode: mode_t) -> Result<()> {
        Sys::mknod(path, mode | S_IFIFO, 0)
    }

    unsafe fn mlock(addr: *const c_void, len: usize) -> Result<()> {
        e_raw(unsafe { syscall!(MLOCK, addr, len) }).map(|_| ())
    }

    unsafe fn mlockall(flags: c_int) -> Result<()> {
        e_raw(unsafe { syscall!(MLOCKALL, flags) }).map(|_| ())
    }

    unsafe fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: c_int,
        flags: c_int,
        fildes: c_int,
        off: off_t,
    ) -> Result<*mut c_void> {
        Ok(e_raw(syscall!(MMAP, addr, len, prot, flags, fildes, off))? as *mut c_void)
    }

    unsafe fn mremap(
        addr: *mut c_void,
        len: usize,
        new_len: usize,
        flags: c_int,
        args: *mut c_void,
    ) -> Result<*mut c_void> {
        Ok(e_raw(syscall!(MREMAP, addr, len, new_len, flags, args))? as *mut c_void)
    }

    unsafe fn mprotect(addr: *mut c_void, len: usize, prot: c_int) -> Result<()> {
        e_raw(syscall!(MPROTECT, addr, len, prot)).map(|_| ())
    }

    unsafe fn msync(addr: *mut c_void, len: usize, flags: c_int) -> Result<()> {
        e_raw(syscall!(MSYNC, addr, len, flags)).map(|_| ())
    }

    unsafe fn munlock(addr: *const c_void, len: usize) -> Result<()> {
        e_raw(syscall!(MUNLOCK, addr, len)).map(|_| ())
    }

    unsafe fn munlockall() -> Result<()> {
        e_raw(unsafe { syscall!(MUNLOCKALL) }).map(|_| ())
    }

    unsafe fn munmap(addr: *mut c_void, len: usize) -> Result<()> {
        e_raw(syscall!(MUNMAP, addr, len)).map(|_| ())
    }

    unsafe fn madvise(addr: *mut c_void, len: usize, flags: c_int) -> Result<()> {
        e_raw(syscall!(MADVISE, addr, len, flags)).map(|_| ())
    }

    unsafe fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> Result<()> {
        e_raw(unsafe { syscall!(NANOSLEEP, rqtp, rmtp) }).map(|_| ())
    }

    fn open(path: CStr, oflag: c_int, mode: mode_t) -> Result<c_int> {
        e_raw(unsafe { syscall!(OPENAT, AT_FDCWD, path.as_ptr(), oflag, mode) })
            .map(|fd| fd as c_int)
    }

    fn pipe2(mut fildes: Out<[c_int; 2]>, flags: c_int) -> Result<()> {
        e_raw(unsafe { syscall!(PIPE2, fildes.as_mut_ptr(), flags) }).map(|_| ())
    }

    fn posix_getdents(fildes: c_int, buf: &mut [u8]) -> Result<usize> {
        let current_offset = Self::lseek(fildes, 0, SEEK_CUR)? as u64;
        let bytes_read = Self::getdents(fildes, buf, current_offset)?;
        if bytes_read == 0 {
            return Ok(0);
        }
        let mut bytes_processed = 0;
        let mut next_offset = current_offset;

        while bytes_processed < bytes_read {
            let remaining_slice = &buf[bytes_processed..];
            let (reclen, opaque_next) =
                unsafe { Self::dent_reclen_offset(remaining_slice, bytes_processed) }
                    .ok_or(Errno(EIO))?;
            if reclen == 0 {
                return Err(Errno(EIO));
            }

            bytes_processed += reclen as usize;
            next_offset = opaque_next;
        }

        Self::lseek(fildes, next_offset as off_t, SEEK_SET)?;
        Ok(bytes_read)
    }

    fn pselect(
        nfds: usize,
        readfds: Option<&mut sys_select::fd_set>,
        writefds: Option<&mut sys_select::fd_set>,
        exceptfds: Option<&mut sys_select::fd_set>,
        timeout: Option<&mut timespec>,
        sigmask: Option<sigset_t>,
    ) -> Result<usize> {
        // The last argument of PSELECT6 isn't a pointer to sigset_t because many architectures
        // only allow passing six args to syscalls. Instead, a pointer to data containing a pointer
        // to sigset_t and the size of the signal set in bytes is used instead.
        // TODO: The calculation below isn't right yet.
        let sigset_data = if let Some(sigmask) = sigmask.as_ref() {
            [
                sigmask as *const sigset_t as size_t,
                size_of::<sigset_t>() as size_t,
            ]
        } else {
            [0, size_of::<sigset_t>() as size_t]
        };
        let nfds: c_int = nfds.try_into().map_err(|_| Errno(EINVAL))?;

        e_raw(unsafe {
            syscall!(
                PSELECT6,
                nfds,
                readfds.map_or(ptr::null_mut(), |mut fds| &mut fds),
                writefds.map_or(ptr::null_mut(), |mut fds| &mut fds),
                exceptfds.map_or(ptr::null_mut(), |mut fds| &mut fds),
                timeout.map_or(ptr::null_mut(), |mut timeout| &mut timeout),
                sigset_data.as_ptr()
            )
        })
    }

    #[cfg(target_arch = "x86_64")]
    unsafe fn rlct_clone(stack: *mut usize) -> Result<crate::pthread::OsTid> {
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
        let tid = e_raw(pid)?;

        Ok(crate::pthread::OsTid { thread_id: tid })
    }
    unsafe fn rlct_kill(os_tid: crate::pthread::OsTid, signal: usize) -> Result<()> {
        let tgid = Self::getpid();
        e_raw(unsafe { syscall!(TGKILL, tgid, os_tid.thread_id, signal) }).map(|_| ())
    }
    fn current_os_tid() -> crate::pthread::OsTid {
        crate::pthread::OsTid {
            thread_id: unsafe { syscall!(GETTID) },
        }
    }

    fn read(fildes: c_int, buf: &mut [u8]) -> Result<usize> {
        e_raw(unsafe { syscall!(READ, fildes, buf.as_mut_ptr(), buf.len()) })
    }
    fn pread(fildes: c_int, buf: &mut [u8], off: off_t) -> Result<usize> {
        e_raw(unsafe { syscall!(PREAD64, fildes, buf.as_mut_ptr(), buf.len(), off) })
    }

    fn readlink(pathname: CStr, out: &mut [u8]) -> Result<usize> {
        e_raw(unsafe {
            syscall!(
                READLINKAT,
                AT_FDCWD,
                pathname.as_ptr(),
                out.as_mut_ptr(),
                out.len()
            )
        })
    }

    fn readlinkat(dirfd: c_int, pathname: CStr, out: &mut [u8]) -> Result<usize> {
        e_raw(unsafe {
            syscall!(
                READLINKAT,
                dirfd,
                pathname.as_ptr(),
                out.as_mut_ptr(),
                out.len()
            )
        })
    }

    fn rename(old: CStr, new: CStr) -> Result<()> {
        e_raw(unsafe { syscall!(RENAMEAT, AT_FDCWD, old.as_ptr(), AT_FDCWD, new.as_ptr()) })
            .map(|_| ())
    }

    fn rmdir(path: CStr) -> Result<()> {
        e_raw(unsafe { syscall!(UNLINKAT, AT_FDCWD, path.as_ptr(), AT_REMOVEDIR) }).map(|_| ())
    }

    fn sched_yield() -> Result<()> {
        e_raw(unsafe { syscall!(SCHED_YIELD) }).map(|_| ())
    }

    unsafe fn setgroups(size: size_t, list: *const gid_t) -> Result<()> {
        e_raw(unsafe { syscall!(SETGROUPS, size, list) }).map(|_| ())
    }

    fn setpgid(pid: pid_t, pgid: pid_t) -> Result<()> {
        e_raw(unsafe { syscall!(SETPGID, pid, pgid) }).map(|_| ())
    }

    fn setpriority(which: c_int, who: id_t, prio: c_int) -> Result<()> {
        e_raw(unsafe { syscall!(SETPRIORITY, which, who, prio) }).map(|_| ())
    }

    fn setresgid(rgid: gid_t, egid: gid_t, sgid: gid_t) -> Result<()> {
        e_raw(unsafe { syscall!(SETRESGID, rgid, egid, sgid) }).map(|_| ())
    }

    fn setresuid(ruid: uid_t, euid: uid_t, suid: uid_t) -> Result<()> {
        e_raw(unsafe { syscall!(SETRESUID, ruid, euid, suid) }).map(|_| ())
    }

    fn setsid() -> Result<c_int> {
        e_raw(unsafe { syscall!(SETSID) }).map(|s| s as c_int)
    }

    fn symlink(path1: CStr, path2: CStr) -> Result<()> {
        e_raw(unsafe { syscall!(SYMLINKAT, path1.as_ptr(), AT_FDCWD, path2.as_ptr()) }).map(|_| ())
    }

    fn sync() -> Result<()> {
        e_raw(unsafe { syscall!(SYNC) }).map(|_| ())
    }

    fn umask(mask: mode_t) -> mode_t {
        unsafe { syscall!(UMASK, mask) as mode_t }
    }

    fn uname(mut utsname: Out<utsname>) -> Result<()> {
        e_raw(unsafe { syscall!(UNAME, utsname.as_mut_ptr(), 0) }).map(|_| ())
    }

    fn unlink(path: CStr) -> Result<()> {
        e_raw(unsafe { syscall!(UNLINKAT, AT_FDCWD, path.as_ptr(), 0) }).map(|_| ())
    }

    fn waitpid(pid: pid_t, stat_loc: Option<Out<c_int>>, options: c_int) -> Result<pid_t> {
        e_raw(unsafe {
            syscall!(
                WAIT4,
                pid,
                stat_loc.map_or(core::ptr::null_mut(), |mut o| o.as_mut_ptr()),
                options,
                0
            )
        })
        .map(|p| p as pid_t)
    }

    fn write(fildes: c_int, buf: &[u8]) -> Result<usize> {
        e_raw(unsafe { syscall!(WRITE, fildes, buf.as_ptr(), buf.len()) })
    }
    fn pwrite(fildes: c_int, buf: &[u8], off: off_t) -> Result<usize> {
        e_raw(unsafe { syscall!(PWRITE64, fildes, buf.as_ptr(), buf.len(), off) })
    }

    fn verify() -> bool {
        // GETPID on Linux is 39, which does not exist on Redox
        e_raw(unsafe { sc::syscall5(sc::nr::GETPID, !0, !0, !0, !0, !0) }).is_ok()
    }
}
