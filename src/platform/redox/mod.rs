use core::{
    convert::TryFrom,
    mem::{self, size_of},
    ptr, slice, str,
};
use redox_rt::{
    protocol::{wifstopped, wstopsig, WaitFlags},
    sys::{Resugid, WaitpidTarget},
    RtTcb,
};
use syscall::{
    self,
    data::{Map, Stat as redox_stat, StatVfs as redox_statvfs, TimeSpec as redox_timespec},
    dirent::{DirentHeader, DirentKind},
    Error, PtraceEvent, EMFILE, MODE_PERM,
};

use crate::{
    c_str::{CStr, CString},
    error::{self, Errno, Result, ResultExt},
    fs::File,
    header::{
        dirent::dirent,
        errno::{
            EBADF, EBADFD, EBADR, EINTR, EINVAL, EIO, ENAMETOOLONG, ENOENT, ENOMEM, ENOSYS,
            EOPNOTSUPP, EPERM, ERANGE,
        },
        fcntl, limits,
        sys_mman::{MAP_ANONYMOUS, MAP_FAILED, PROT_READ, PROT_WRITE},
        sys_random,
        sys_resource::{rlimit, rusage, RLIM_INFINITY},
        sys_stat::{stat, S_ISGID, S_ISUID, S_ISVTX},
        sys_statvfs::statvfs,
        sys_time::{timeval, timezone},
        sys_utsname::{utsname, UTSLENGTH},
        sys_wait,
        time::timespec,
        unistd::{F_OK, R_OK, W_OK, X_OK},
    },
    io::{self, prelude::*, BufReader},
    sync::rwlock::RwLock,
};

pub use redox_rt::proc::FdGuard;

use super::{types::*, Pal, Read, ERRNO};

static mut BRK_CUR: *mut c_void = ptr::null_mut();
static mut BRK_END: *mut c_void = ptr::null_mut();

const PAGE_SIZE: usize = 4096;
fn round_up_to_page_size(val: usize) -> Option<usize> {
    val.checked_add(PAGE_SIZE)
        .map(|val| (val - 1) / PAGE_SIZE * PAGE_SIZE)
}

fn cvt_uid(id: c_int) -> Result<Option<u32>> {
    if id == -1 {
        return Ok(None);
    }
    Ok(Some(id.try_into().map_err(|_| Errno(EINVAL))?))
}

mod clone;
mod epoll;
mod event;
mod exec;
mod extra;
mod libcscheme;
mod libredox;
pub(crate) mod path;
mod ptrace;
pub(crate) mod signal;
mod socket;

macro_rules! path_from_c_str {
    ($c_str:expr) => {{
        match $c_str.to_str() {
            Ok(ok) => ok,
            Err(err) => {
                ERRNO.set(EINVAL);
                return -1;
            }
        }
    }};
}

use self::{exec::Executable, path::canonicalize};

static CLONE_LOCK: RwLock<()> = RwLock::new(());

/// Redox syscall implementation of the platform abstraction layer.
pub struct Sys;

impl Pal for Sys {
    fn access(path: CStr, mode: c_int) -> Result<()> {
        let fd = FdGuard::new(Sys::open(path, fcntl::O_PATH | fcntl::O_CLOEXEC, 0)? as usize);

        if mode == F_OK {
            return Ok(());
        }

        let mut stat = syscall::Stat::default();

        syscall::fstat(*fd as usize, &mut stat)?;

        let Resugid { ruid, rgid, .. } = redox_rt::sys::posix_getresugid();

        let perms = if stat.st_uid == ruid {
            stat.st_mode >> (3 * 2 & 0o7)
        } else if stat.st_gid == rgid {
            stat.st_mode >> (3 * 1 & 0o7)
        } else {
            stat.st_mode & 0o7
        };
        if (mode & R_OK == R_OK && perms & 0o4 != 0o4)
            || (mode & W_OK == W_OK && perms & 0o2 != 0o2)
            || (mode & X_OK == X_OK && perms & 0o1 != 0o1)
        {
            return Err(Errno(EINVAL));
        }

        Ok(())
    }

    unsafe fn brk(addr: *mut c_void) -> Result<*mut c_void> {
        // On first invocation, allocate a buffer for brk
        if BRK_CUR.is_null() {
            // 4 megabytes of RAM ought to be enough for anybody
            const BRK_MAX_SIZE: usize = 4 * 1024 * 1024;

            let allocated = Self::mmap(
                ptr::null_mut(),
                BRK_MAX_SIZE,
                PROT_READ | PROT_WRITE,
                MAP_ANONYMOUS,
                0,
                0,
            )?;

            BRK_CUR = allocated;
            BRK_END = (allocated as *mut u8).add(BRK_MAX_SIZE) as *mut c_void;
        }

        if addr.is_null() {
            // Lookup what previous brk() invocations have set the address to
            Ok(BRK_CUR)
        } else if BRK_CUR <= addr && addr < BRK_END {
            // It's inside buffer, return
            BRK_CUR = addr;
            Ok(addr)
        } else {
            // It was outside of valid range
            Err(Errno(ENOMEM))
        }
    }

    fn chdir(path: CStr) -> Result<()> {
        let path = path.to_str().map_err(|_| Errno(EINVAL))?;
        path::chdir(path)?;
        Ok(())
    }
    fn set_default_scheme(path: CStr) -> Result<()> {
        let path = path.to_str().map_err(|_| Errno(EINVAL))?;
        Ok(path::set_default_scheme(path)?)
    }

    fn chmod(path: CStr, mode: mode_t) -> Result<()> {
        let file = File::open(path, fcntl::O_PATH | fcntl::O_CLOEXEC)?;
        Self::fchmod(*file, mode)
    }

    fn chown(path: CStr, owner: uid_t, group: gid_t) -> Result<()> {
        let file = File::open(path, fcntl::O_PATH | fcntl::O_CLOEXEC)?;
        Self::fchown(*file, owner, group)
    }

    unsafe fn clock_getres(clk_id: clockid_t, tp: *mut timespec) -> Result<()> {
        // TODO
        eprintln!("relibc clock_getres({}, {:p}): not implemented", clk_id, tp);
        Err(Errno(ENOSYS))
    }

    unsafe fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> Result<()> {
        libredox::clock_gettime(clk_id as usize, tp)?;
        Ok(())
    }

    unsafe fn clock_settime(clk_id: clockid_t, tp: *const timespec) -> Result<()> {
        // TODO
        eprintln!(
            "relibc clock_settime({}, {:p}): not implemented",
            clk_id, tp
        );
        Err(Errno(ENOSYS))
    }

    fn close(fd: c_int) -> Result<()> {
        syscall::close(fd as usize)?;
        Ok(())
    }

    fn dup(fd: c_int) -> Result<c_int> {
        Ok(syscall::dup(fd as usize, &[])? as c_int)
    }

    fn dup2(fd1: c_int, fd2: c_int) -> Result<c_int> {
        Ok(syscall::dup2(fd1 as usize, fd2 as usize, &[])? as c_int)
    }

    fn exit(status: c_int) -> ! {
        let _ = redox_rt::sys::posix_exit(status);
        loop {}
    }

    unsafe fn execve(path: CStr, argv: *const *mut c_char, envp: *const *mut c_char) -> Result<()> {
        self::exec::execve(
            Executable::AtPath(path),
            self::exec::ArgEnv::C { argv, envp },
            None,
        )?;
        unreachable!()
    }
    unsafe fn fexecve(
        fildes: c_int,
        argv: *const *mut c_char,
        envp: *const *mut c_char,
    ) -> Result<()> {
        self::exec::execve(
            Executable::InFd {
                file: File::new(fildes),
                arg0: CStr::from_ptr(argv.read()).to_bytes(),
            },
            self::exec::ArgEnv::C { argv, envp },
            None,
        )?;
        unreachable!()
    }

    fn fchdir(fd: c_int) -> Result<()> {
        let mut buf = [0; 4096];
        let res = syscall::fpath(fd as usize, &mut buf)?;

        let path = str::from_utf8(&buf[..res]).map_err(|_| Errno(EINVAL))?;
        path::chdir(path)?;
        Ok(())
    }

    fn fchmod(fd: c_int, mode: mode_t) -> Result<()> {
        syscall::fchmod(fd as usize, mode as u16)?;
        Ok(())
    }

    fn fchown(fd: c_int, owner: uid_t, group: gid_t) -> Result<()> {
        syscall::fchown(fd as usize, owner as u32, group as u32)?;
        Ok(())
    }

    fn fcntl(fd: c_int, cmd: c_int, args: c_ulonglong) -> Result<c_int> {
        Ok(syscall::fcntl(fd as usize, cmd as usize, args as usize)? as c_int)
    }

    fn fdatasync(fd: c_int) -> Result<()> {
        // TODO: "Needs" syscall update
        syscall::fsync(fd as usize)?;
        Ok(())
    }

    fn flock(_fd: c_int, _operation: c_int) -> Result<()> {
        // TODO: Redox does not have file locking yet
        Ok(())
    }

    unsafe fn fork() -> Result<pid_t> {
        // TODO: Find way to avoid lock.
        let _guard = CLONE_LOCK.write();

        Ok(clone::fork_impl(&redox_rt::proc::ForkArgs::Managed)? as pid_t)
    }

    unsafe fn fstat(fildes: c_int, buf: *mut stat) -> Result<()> {
        libredox::fstat(fildes as usize, buf)?;
        Ok(())
    }

    unsafe fn fstatvfs(fildes: c_int, buf: *mut statvfs) -> Result<()> {
        libredox::fstatvfs(fildes as usize, buf)?;
        Ok(())
    }

    fn fsync(fd: c_int) -> Result<()> {
        syscall::fsync(fd as usize)?;
        Ok(())
    }

    fn ftruncate(fd: c_int, len: off_t) -> Result<()> {
        syscall::ftruncate(fd as usize, len as usize)?;
        Ok(())
    }

    #[inline]
    unsafe fn futex_wait(addr: *mut u32, val: u32, deadline: Option<&timespec>) -> Result<()> {
        let deadline = deadline.map(|d| syscall::TimeSpec {
            tv_sec: d.tv_sec,
            tv_nsec: d.tv_nsec as i32,
        });
        redox_rt::sys::sys_futex_wait(addr, val, deadline.as_ref())?;
        Ok(())
    }
    #[inline]
    unsafe fn futex_wake(addr: *mut u32, num: u32) -> Result<u32> {
        Ok(redox_rt::sys::sys_futex_wake(addr, num)?)
    }

    unsafe fn futimens(fd: c_int, times: *const timespec) -> Result<()> {
        libredox::futimens(fd as usize, times)?;
        Ok(())
    }

    unsafe fn utimens(path: CStr, times: *const timespec) -> Result<()> {
        let file = File::open(path, fcntl::O_PATH | fcntl::O_CLOEXEC)?;
        Self::futimens(*file, times)
    }

    unsafe fn getcwd(buf: *mut c_char, size: size_t) -> Result<()> {
        // TODO: Not using MaybeUninit seems a little unsafe

        let buf_slice = unsafe { slice::from_raw_parts_mut(buf as *mut u8, size as usize) };
        if buf_slice.is_empty() {
            return Err(Errno(EINVAL));
        }

        path::getcwd(buf_slice).ok_or(Errno(ERANGE))?;
        Ok(())
    }

    fn getdents(fd: c_int, buf: &mut [u8], opaque: u64) -> Result<usize> {
        //println!("GETDENTS {} into ({:p}+{})", fd, buf.as_ptr(), buf.len());

        const HEADER_SIZE: usize = size_of::<DirentHeader>();

        // Use syscall if it exists.
        match unsafe {
            syscall::syscall5(
                syscall::SYS_GETDENTS,
                fd as usize,
                buf.as_mut_ptr() as usize,
                buf.len(),
                HEADER_SIZE,
                opaque as usize,
            )
        } {
            Err(Error {
                errno: EOPNOTSUPP | ENOSYS,
            }) => (),
            other => {
                //println!("REAL GETDENTS {:?}", other);
                return Ok(other?);
            }
        }

        // Otherwise, for legacy schemes, assume the buffer is pre-arranged (all schemes do this in
        // practice), and just read the name. If multiple names appear, pretend it didn't happen
        // and just use the first entry.

        let (header, name) = buf.split_at_mut(size_of::<DirentHeader>());

        let bytes_read = Sys::pread(fd, name, opaque as i64)? as usize;
        if bytes_read == 0 {
            return Ok(0);
        }

        let (name_len, advance) = match name[..bytes_read].iter().position(|c| *c == b'\n') {
            Some(idx) => (idx, idx + 1),

            // Insufficient space for NUL byte, or entire entry was not read. Indicate we need a
            // larger buffer.
            None if bytes_read == name.len() => return Err(Errno(EINVAL)),

            None => (bytes_read, name.len()),
        };
        name[name_len] = b'\0';

        let record_len = u16::try_from(size_of::<DirentHeader>() + name_len + 1)
            .map_err(|_| Error::new(ENAMETOOLONG))?;
        header.copy_from_slice(&DirentHeader {
            inode: 0,
            next_opaque_id: opaque + advance as u64,
            record_len,
            kind: DirentKind::Unspecified as u8,
        });
        //println!("EMULATED GETDENTS");

        Ok(record_len.into())
    }
    fn dir_seek(_fd: c_int, _off: u64) -> Result<()> {
        // Redox getdents takes an explicit (opaque) offset, so this is a no-op.
        Ok(())
    }
    // NOTE: fn is unsafe, but this just means we can assume more things. impl is safe
    unsafe fn dent_reclen_offset(this_dent: &[u8], offset: usize) -> Option<(u16, u64)> {
        let mut header = DirentHeader::default();
        header.copy_from_slice(&this_dent.get(..size_of::<DirentHeader>())?);

        // If scheme does not send a NUL byte, this shouldn't be able to cause UB for the caller.
        if this_dent.get(usize::from(header.record_len) - 1) != Some(&b'\0') {
            return None;
        }

        Some((header.record_len, header.next_opaque_id))
    }

    fn getegid() -> gid_t {
        redox_rt::sys::posix_getresugid().egid as gid_t
    }

    fn geteuid() -> uid_t {
        redox_rt::sys::posix_getresugid().euid as uid_t
    }

    fn getgid() -> gid_t {
        redox_rt::sys::posix_getresugid().rgid as gid_t
    }

    unsafe fn getgroups(size: c_int, list: *mut gid_t) -> Result<c_int> {
        // TODO
        eprintln!("relibc getgroups({}, {:p}): not implemented", size, list);
        Err(Errno(ENOSYS))
    }

    fn getpagesize() -> usize {
        PAGE_SIZE
    }

    fn getpgid(pid: pid_t) -> Result<pid_t> {
        Ok(redox_rt::sys::posix_getpgid(pid as usize)? as pid_t)
    }

    fn getpid() -> pid_t {
        redox_rt::sys::posix_getpid() as pid_t
    }

    fn getppid() -> pid_t {
        redox_rt::sys::posix_getppid() as pid_t
    }

    fn getpriority(which: c_int, who: id_t) -> Result<c_int> {
        // TODO
        eprintln!("getpriority({}, {}): not implemented", which, who);
        Err(Errno(ENOSYS))
    }

    fn getrandom(buf: &mut [u8], flags: c_uint) -> Result<usize> {
        let path = if flags & sys_random::GRND_RANDOM != 0 {
            //TODO: /dev/random equivalent
            "/scheme/rand"
        } else {
            "/scheme/rand"
        };

        let mut open_flags = syscall::O_RDONLY | syscall::O_CLOEXEC;
        if flags & sys_random::GRND_NONBLOCK != 0 {
            open_flags |= syscall::O_NONBLOCK;
        }

        //TODO: store fd internally
        let fd = FdGuard::new(syscall::open(path, open_flags)?);
        Ok(syscall::read(*fd, buf)?)
    }

    fn getresgid(
        rgid_out: Option<&mut gid_t>,
        egid_out: Option<&mut gid_t>,
        sgid_out: Option<&mut gid_t>,
    ) -> Result<()> {
        let Resugid {
            rgid, egid, sgid, ..
        } = redox_rt::sys::posix_getresugid();
        if let Some(rgid_out) = rgid_out {
            *rgid_out = rgid as _;
        }
        if let Some(egid_out) = egid_out {
            *egid_out = egid as _;
        }
        if let Some(sgid_out) = sgid_out {
            *sgid_out = sgid as _;
        }
        Ok(())
    }
    fn getresuid(
        ruid_out: Option<&mut uid_t>,
        euid_out: Option<&mut uid_t>,
        suid_out: Option<&mut uid_t>,
    ) -> Result<()> {
        let Resugid {
            ruid, euid, suid, ..
        } = redox_rt::sys::posix_getresugid();
        if let Some(ruid_out) = ruid_out {
            *ruid_out = ruid as _;
        }
        if let Some(euid_out) = euid_out {
            *euid_out = euid as _;
        }
        if let Some(suid_out) = suid_out {
            *suid_out = suid as _;
        }
        Ok(())
    }

    unsafe fn getrlimit(resource: c_int, rlim: *mut rlimit) -> Result<()> {
        //TODO
        eprintln!(
            "relibc getrlimit({}, {:p}): not implemented",
            resource, rlim
        );
        if !rlim.is_null() {
            (*rlim).rlim_cur = RLIM_INFINITY;
            (*rlim).rlim_max = RLIM_INFINITY;
        }
        Ok(())
    }

    unsafe fn setrlimit(resource: c_int, rlim: *const rlimit) -> Result<()> {
        //TOOD
        eprintln!(
            "relibc setrlimit({}, {:p}): not implemented",
            resource, rlim
        );
        Err(Errno(EPERM))
    }

    fn getrusage(who: c_int, r_usage: &mut rusage) -> Result<()> {
        //TODO
        eprintln!("relibc getrusage({}, {:p}): not implemented", who, r_usage);
        Ok(())
    }

    fn getsid(pid: pid_t) -> Result<pid_t> {
        Ok(redox_rt::sys::posix_getsid(pid as usize)? as _)
    }

    fn gettid() -> pid_t {
        // This is used by pthread mutexes for reentrant checks and must be nonzero
        // and unique for each thread in the same process (but not cross-process)
        Self::current_os_tid()
            .thread_fd
            .checked_add(1)
            .unwrap()
            .try_into()
            .unwrap()
    }

    unsafe fn gettimeofday(tp: *mut timeval, tzp: *mut timezone) -> Result<()> {
        let mut redox_tp = redox_timespec::default();
        syscall::clock_gettime(syscall::CLOCK_REALTIME, &mut redox_tp)?;
        unsafe {
            (*tp).tv_sec = redox_tp.tv_sec as time_t;
            (*tp).tv_usec = (redox_tp.tv_nsec / 1000) as suseconds_t;

            if !tzp.is_null() {
                (*tzp).tz_minuteswest = 0;
                (*tzp).tz_dsttime = 0;
            }
        }
        Ok(())
    }

    fn getuid() -> uid_t {
        redox_rt::sys::posix_getresugid().ruid as uid_t
    }

    fn lchown(path: CStr, owner: uid_t, group: gid_t) -> Result<()> {
        // TODO: Is it correct for regular chown to use O_PATH? On Linux the meaning of that flag
        // is to forbid file operations, including fchown.

        // unlike chown, never follow symbolic links
        let file = File::open(path, fcntl::O_CLOEXEC | fcntl::O_NOFOLLOW)?;
        Self::fchown(*file, owner, group)
    }

    fn link(path1: CStr, path2: CStr) -> Result<()> {
        unsafe { syscall::link(path1.as_ptr() as *const u8, path2.as_ptr() as *const u8)? };
        Ok(())
    }

    fn lseek(fd: c_int, offset: off_t, whence: c_int) -> Result<off_t> {
        Ok(syscall::lseek(fd as usize, offset as isize, whence as usize)? as off_t)
    }

    fn mkdir(path: CStr, mode: mode_t) -> Result<()> {
        File::create(
            path,
            fcntl::O_DIRECTORY | fcntl::O_EXCL | fcntl::O_CLOEXEC,
            0o777,
        )?;
        Ok(())
    }

    fn mkfifo(path: CStr, mode: mode_t) -> Result<()> {
        Sys::mknod(path, syscall::MODE_FIFO as mode_t | (mode & 0o777), 0)
    }

    fn mknodat(dir_fd: c_int, path_name: CStr, mode: mode_t, dev: dev_t) -> Result<()> {
        let mut dir_path_buf = [0; 4096];
        let res = Sys::fpath(dir_fd, &mut dir_path_buf)?;

        let dir_path = str::from_utf8(&dir_path_buf[..res as usize]).map_err(|_| Errno(EBADR))?;

        let resource_path =
            path::canonicalize_using_cwd(Some(&dir_path), &path_name.to_string_lossy())
                // Since parent_dir_path is resolved by fpath, it is more likely that
                // the problem was with path.
                .ok_or(Errno(ENOENT))?;

        Sys::mknod(
            CStr::borrow(&CString::new(resource_path.as_bytes()).unwrap()),
            mode,
            dev,
        )
    }

    fn mknod(path: CStr, mode: mode_t, dev: dev_t) -> Result<(), Errno> {
        File::create(path, fcntl::O_CREAT | fcntl::O_CLOEXEC, mode)?;
        Ok(())
    }

    unsafe fn mlock(addr: *const c_void, len: usize) -> Result<()> {
        // Redox never swaps
        Ok(())
    }

    unsafe fn mlockall(flags: c_int) -> Result<()> {
        // Redox never swaps
        Ok(())
    }

    unsafe fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: c_int,
        flags: c_int,
        fildes: c_int,
        off: off_t,
    ) -> Result<*mut c_void> {
        // 0 is invalid per spec
        if len == 0 {
            return Err(Errno(EINVAL));
        }
        let Some(size) = round_up_to_page_size(len) else {
            return Err(Errno(ENOMEM));
        };

        let map = Map {
            offset: off as usize,
            size,
            flags: syscall::MapFlags::from_bits_truncate(
                ((prot as usize) << 16) | ((flags as usize) & 0xFFFF),
            ),
            address: addr as usize,
        };

        Ok(if flags & MAP_ANONYMOUS == MAP_ANONYMOUS {
            syscall::fmap(!0, &map)?
        } else {
            syscall::fmap(fildes as usize, &map)?
        } as *mut c_void)
    }

    unsafe fn mremap(
        addr: *mut c_void,
        len: usize,
        new_len: usize,
        flags: c_int,
        args: *mut c_void,
    ) -> Result<*mut c_void> {
        Err(Errno(ENOSYS))
    }

    unsafe fn mprotect(addr: *mut c_void, len: usize, prot: c_int) -> Result<()> {
        let Some(len) = round_up_to_page_size(len) else {
            return Err(Errno(ENOMEM));
        };
        syscall::mprotect(
            addr as usize,
            len,
            syscall::MapFlags::from_bits((prot as usize) << 16)
                .expect("mprotect: invalid bit pattern"),
        )?;
        Ok(())
    }

    unsafe fn msync(addr: *mut c_void, len: usize, flags: c_int) -> Result<()> {
        eprintln!(
            "relibc msync({:p}, 0x{:x}, 0x{:x}): not implemented",
            addr, len, flags
        );
        Err(Errno(ENOSYS))
        /* TODO
        syscall::msync(
            addr as usize,
            round_up_to_page_size(len),
            flags
        )?;
        */
    }

    unsafe fn munlock(addr: *const c_void, len: usize) -> Result<()> {
        // Redox never swaps
        Ok(())
    }

    unsafe fn munlockall() -> Result<()> {
        // Redox never swaps
        Ok(())
    }

    unsafe fn munmap(addr: *mut c_void, len: usize) -> Result<()> {
        // 0 is invalid per spec
        if len == 0 {
            return Err(Errno(EINVAL));
        }
        let Some(len) = round_up_to_page_size(len) else {
            return Err(Errno(ENOMEM));
        };
        syscall::funmap(addr as usize, len)?;
        Ok(())
    }

    unsafe fn madvise(addr: *mut c_void, len: usize, flags: c_int) -> Result<()> {
        eprintln!(
            "relibc madvise({:p}, 0x{:x}, 0x{:x}): not implemented",
            addr, len, flags
        );
        Err(Errno(ENOSYS))
    }

    unsafe fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> Result<()> {
        let redox_rqtp = unsafe { redox_timespec::from(&*rqtp) };
        let mut redox_rmtp: redox_timespec;
        if rmtp.is_null() {
            redox_rmtp = redox_timespec::default();
        } else {
            redox_rmtp = unsafe { redox_timespec::from(&*rmtp) };
        }
        match redox_rt::sys::posix_nanosleep(&redox_rqtp, &mut redox_rmtp) {
            Ok(_) => Ok(()),
            Err(Error { errno: EINTR }) => {
                unsafe {
                    if !rmtp.is_null() {
                        (*rmtp).tv_sec = redox_rmtp.tv_sec as time_t;
                        (*rmtp).tv_nsec = redox_rmtp.tv_nsec as c_long;
                    }
                };
                Err(Errno(EINTR))
            }
            Err(Error { errno: e }) => Err(Errno(e)),
        }
    }

    fn open(path: CStr, oflag: c_int, mode: mode_t) -> Result<c_int> {
        let path = path.to_str().map_err(|_| Errno(EINVAL))?;

        // POSIX states that umask should affect the following:
        //
        // open, openat (TODO), creat, mkdir, mkdirat (TODO),
        // mkfifo, mkfifoat (TODO), mknod, mknodat (TODO),
        // mq_open, and sem_open,
        //
        // all of which (the ones that exist thus far) currently call this function.
        let effective_mode = mode & !(redox_rt::sys::get_umask() as mode_t);

        Ok(libredox::open(path, oflag, effective_mode)? as c_int)
    }

    fn pipe2(fds: &mut [c_int], flags: c_int) -> Result<()> {
        extra::pipe2(fds, flags as usize)?;
        Ok(())
    }

    unsafe fn rlct_clone(stack: *mut usize) -> Result<crate::pthread::OsTid> {
        let _guard = CLONE_LOCK.read();
        let res = clone::rlct_clone_impl(stack);

        res.map(|mut fd| crate::pthread::OsTid {
            thread_fd: fd.take(),
        })
        .map_err(|error| Errno(error.errno))
    }
    unsafe fn rlct_kill(os_tid: crate::pthread::OsTid, signal: usize) -> Result<()> {
        redox_rt::sys::posix_kill_thread(os_tid.thread_fd, signal as u32)?;
        Ok(())
    }
    fn current_os_tid() -> crate::pthread::OsTid {
        crate::pthread::OsTid {
            thread_fd: **RtTcb::current().thread_fd(),
        }
    }

    fn read(fd: c_int, buf: &mut [u8]) -> Result<usize> {
        let fd = usize::try_from(fd).map_err(|_| Errno(EBADF))?;
        Ok(redox_rt::sys::posix_read(fd, buf)?)
    }
    fn pread(fd: c_int, buf: &mut [u8], offset: off_t) -> Result<usize> {
        unsafe {
            Ok(syscall::syscall5(
                syscall::SYS_READ2,
                fd as usize,
                buf.as_mut_ptr() as usize,
                buf.len(),
                offset as usize,
                !0,
            )?)
        }
    }

    fn fpath(fildes: c_int, out: &mut [u8]) -> Result<usize> {
        // Since this is used by realpath, it converts from the old format to the new one for
        // compatibility reasons
        let mut buf = [0; limits::PATH_MAX];
        let count = syscall::fpath(fildes as usize, &mut buf)?;

        let redox_path = str::from_utf8(&buf[..count])
            .ok()
            .and_then(|x| redox_path::RedoxPath::from_absolute(x))
            .ok_or(Errno(EINVAL))?;

        let (scheme, reference) = redox_path.as_parts().ok_or(Errno(EINVAL))?;

        let mut cursor = io::Cursor::new(out);
        let res = match scheme.as_ref() {
            "file" => write!(cursor, "/{}", reference.as_ref().trim_start_matches('/')),
            _ => write!(
                cursor,
                "/scheme/{}/{}",
                scheme.as_ref(),
                reference.as_ref().trim_start_matches('/')
            ),
        };
        match res {
            Ok(()) => Ok(cursor.position() as usize),
            Err(_err) => Err(Errno(ENAMETOOLONG)),
        }
    }

    fn readlink(pathname: CStr, out: &mut [u8]) -> Result<usize> {
        let file = File::open(
            pathname,
            fcntl::O_RDONLY | fcntl::O_SYMLINK | fcntl::O_CLOEXEC,
        )?;
        Self::read(*file, out)
    }

    fn rename(oldpath: CStr, newpath: CStr) -> Result<()> {
        let newpath = newpath.to_str().map_err(|_| Errno(EINVAL))?;

        let file = File::open(oldpath, fcntl::O_PATH | fcntl::O_CLOEXEC)?;
        syscall::frename(*file as usize, newpath)?;
        Ok(())
    }

    fn rmdir(path: CStr) -> Result<()> {
        let path = path.to_str().map_err(|_| Errno(EINVAL))?;
        let canon = canonicalize(path)?;
        syscall::rmdir(&canon)?;
        Ok(())
    }

    fn sched_yield() -> Result<()> {
        syscall::sched_yield()?;
        Ok(())
    }

    unsafe fn setgroups(size: size_t, list: *const gid_t) -> Result<()> {
        // TODO
        eprintln!("relibc setgroups({}, {:p}): not implemented", size, list);
        Err(Errno(ENOSYS))
    }

    fn setpgid(pid: pid_t, pgid: pid_t) -> Result<()> {
        redox_rt::sys::posix_setpgid(pid as usize, pgid as usize)?;
        Ok(())
    }

    fn setpriority(which: c_int, who: id_t, prio: c_int) -> Result<()> {
        // TODO
        eprintln!(
            "relibc setpriority({}, {}, {}): not implemented",
            which, who, prio
        );
        Err(Errno(ENOSYS))
    }

    fn setsid() -> Result<c_int> {
        Ok(redox_rt::sys::posix_setsid()? as c_int)
    }

    fn setresgid(rgid: gid_t, egid: gid_t, sgid: gid_t) -> Result<()> {
        redox_rt::sys::posix_setresugid(&Resugid {
            ruid: None,
            euid: None,
            suid: None,
            rgid: cvt_uid(rgid)?,
            egid: cvt_uid(egid)?,
            sgid: cvt_uid(sgid)?,
        })?;
        Ok(())
    }

    fn setresuid(ruid: uid_t, euid: uid_t, suid: uid_t) -> Result<()> {
        redox_rt::sys::posix_setresugid(&Resugid {
            ruid: cvt_uid(ruid)?,
            euid: cvt_uid(euid)?,
            suid: cvt_uid(suid)?,
            rgid: None,
            egid: None,
            sgid: None,
        })?;
        Ok(())
    }

    fn symlink(path1: CStr, path2: CStr) -> Result<()> {
        let mut file = File::create(
            path2,
            fcntl::O_WRONLY | fcntl::O_SYMLINK | fcntl::O_CLOEXEC,
            0o777,
        )?;

        file.write(path1.to_bytes())
            .map_err(|err| Errno(err.raw_os_error().unwrap_or(EIO)))?;

        Ok(())
    }

    fn sync() -> Result<()> {
        Ok(())
    }

    fn umask(mask: mode_t) -> mode_t {
        let new_effective_mask = mask & mode_t::from(MODE_PERM) & !S_ISVTX;
        (redox_rt::sys::swap_umask(new_effective_mask as u32) as mode_t) & !S_ISVTX
    }

    unsafe fn uname(utsname: *mut utsname) -> Result<(), Errno> {
        fn gethostname(name: &mut [u8]) -> io::Result<()> {
            if name.is_empty() {
                return Ok(());
            }

            let mut file = File::open(c"/etc/hostname".into(), fcntl::O_RDONLY | fcntl::O_CLOEXEC)?;

            let mut read = 0;
            let name_len = name.len();
            loop {
                match file.read(&mut name[read..name_len - 1])? {
                    0 => break,
                    n => read += n,
                }
            }
            name[read] = 0;
            Ok(())
        }

        match gethostname(unsafe {
            slice::from_raw_parts_mut(
                (*utsname).nodename.as_mut_ptr() as *mut u8,
                (*utsname).nodename.len(),
            )
        }) {
            Ok(_) => (),
            Err(_) => return Err(Errno(EIO)),
        }

        let file_path = c"/scheme/sys/uname".into();
        let mut file = match File::open(file_path, fcntl::O_RDONLY | fcntl::O_CLOEXEC) {
            Ok(ok) => ok,
            Err(_) => return Err(Errno(EIO)),
        };
        let mut lines = BufReader::new(&mut file).lines();

        let mut read_line = |dst: &mut [c_char]| {
            let line = match lines.next() {
                Some(Ok(l)) => match CString::new(l) {
                    Ok(l) => l,
                    Err(_) => return Err(Errno(EIO)),
                },
                None | Some(Err(_)) => return Err(Errno(EIO)),
            };

            let line_slice: &[c_char] = unsafe { mem::transmute(line.as_bytes_with_nul()) };

            if line_slice.len() <= UTSLENGTH {
                dst[..line_slice.len()].copy_from_slice(line_slice);
                Ok(())
            } else {
                Err(Errno(EIO))
            }
        };

        unsafe {
            read_line(&mut (*utsname).sysname)?;
            read_line(&mut (*utsname).release)?;
            read_line(&mut (*utsname).machine)?;

            // Version is not provided
            ptr::write_bytes((*utsname).version.as_mut_ptr(), 0, UTSLENGTH);

            // Redox doesn't provide domainname in sys:uname
            //read_line(&mut (*utsname).domainname)?;
            ptr::write_bytes((*utsname).domainname.as_mut_ptr(), 0, UTSLENGTH);
        }

        Ok(())
    }

    fn unlink(path: CStr) -> Result<()> {
        let path = path.to_str().map_err(|_| Errno(EINVAL))?;
        let canon = canonicalize(path)?;
        syscall::unlink(&canon)?;
        Ok(())
    }

    unsafe fn waitpid(mut pid: pid_t, stat_loc: *mut c_int, options: c_int) -> Result<pid_t> {
        let mut res = None;
        let mut status = 0;

        let options = usize::try_from(options)
            .ok()
            .and_then(WaitFlags::from_bits)
            .ok_or(Errno(EINVAL))?;

        let inner = |status: &mut usize, flags| {
            redox_rt::sys::sys_waitpid(WaitpidTarget::from_posix_arg(pid as isize), status, flags)
        };

        // First, allow ptrace to handle waitpid
        // TODO: Handle special PIDs here (such as -1)
        let state = ptrace::init_state();
        let mut sessions = state.sessions.lock();
        if let Ok(session) = ptrace::get_session(&mut sessions, pid) {
            if !options.contains(WaitFlags::WNOHANG) {
                let mut _event = PtraceEvent::default();
                let _ = (&mut &session.tracer).read(&mut _event);

                res = Some(inner(
                    &mut status,
                    options | WaitFlags::WNOHANG | WaitFlags::WUNTRACED,
                ));
                if res == Some(Ok(0)) {
                    // WNOHANG, just pretend ptrace SIGSTOP:ped this
                    status = (redox_rt::protocol::SIGSTOP << 8) | 0x7f;
                    assert!(wifstopped(status));
                    assert_eq!(wstopsig(status), redox_rt::protocol::SIGSTOP);
                    res = Some(Ok(pid as usize));
                }
            }
        }

        // If ptrace didn't impact this waitpid, proceed *almost* as
        // normal: We still need to add WUNTRACED, but we only return
        // it if (and only if) a ptrace traceme was activated during
        // the wait.
        let res = res.unwrap_or_else(|| loop {
            let res = inner(&mut status, options | WaitFlags::WUNTRACED);

            // TODO: Also handle special PIDs here
            if !wifstopped(status)
                || options.contains(WaitFlags::WUNTRACED)
                || ptrace::is_traceme(pid)
            {
                break res;
            }
        });

        // If stat_loc is non-null, set that and the return
        unsafe {
            if !stat_loc.is_null() {
                *stat_loc = status as c_int;
            }
        }
        Ok(res? as pid_t)
    }

    fn write(fd: c_int, buf: &[u8]) -> Result<usize> {
        let fd = usize::try_from(fd).map_err(|_| Errno(EBADFD))?;
        Ok(redox_rt::sys::posix_write(fd, buf)?)
    }
    fn pwrite(fd: c_int, buf: &[u8], offset: off_t) -> Result<usize> {
        unsafe {
            Ok(syscall::syscall5(
                syscall::SYS_WRITE2,
                fd as usize,
                buf.as_ptr() as usize,
                buf.len(),
                offset as usize,
                !0,
            )?)
        }
    }

    fn verify() -> bool {
        // YIELD on Redox is 20, which is SYS_ARCH_PRCTL on Linux
        (unsafe { syscall::syscall5(syscall::number::SYS_YIELD, !0, !0, !0, !0, !0) }).is_ok()
    }

    unsafe fn exit_thread(stack_base: *mut (), stack_size: usize) -> ! {
        redox_rt::thread::exit_this_thread(stack_base, stack_size)
    }
}
