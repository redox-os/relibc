use core::{convert::TryFrom, mem, ptr, result::Result as CoreResult, slice, str};

use syscall::{
    self,
    data::{Map, Stat as redox_stat, StatVfs as redox_statvfs, TimeSpec as redox_timespec},
    Error, PtraceEvent, Result, EMFILE,
};

use crate::{
    c_str::{CStr, CString},
    errno::Errno,
    fs::File,
    header::{
        dirent::dirent,
        errno::{EINVAL, EIO, ENOMEM, ENOSYS, EPERM, ERANGE},
        fcntl,
        sys_mman::{MAP_ANONYMOUS, PROT_READ, PROT_WRITE},
        sys_random,
        sys_resource::{rlimit, RLIM_INFINITY},
        sys_stat::{stat, S_ISGID, S_ISUID},
        sys_statvfs::statvfs,
        sys_time::{timeval, timezone},
        sys_utsname::{utsname, UTSLENGTH},
        sys_wait,
        time::timespec,
        unistd::{F_OK, R_OK, W_OK, X_OK},
    },
    io::{self, prelude::*, BufReader},
};

pub use redox_exec::FdGuard;

use super::{errno, types::*, Pal, Read};

static mut BRK_CUR: *mut c_void = ptr::null_mut();
static mut BRK_END: *mut c_void = ptr::null_mut();

const PAGE_SIZE: usize = 4096;
fn round_up_to_page_size(val: usize) -> usize {
    (val + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE
}

mod clone;
mod epoll;
mod exec;
mod extra;
mod libredox;
pub(crate) mod path;
mod ptrace;
mod signal;
mod socket;

macro_rules! path_from_c_str {
    ($c_str:expr) => {{
        $c_str.to_str().map_err(|_| Errno(EINVAL))
    }};
}

use self::{exec::Executable, path::canonicalize};

pub fn e_raw(sys: Result<usize>) -> Result<usize, Errno> {
    sys.map_err(|err| Errno(err.errno as c_int))
}

pub struct Sys;

impl Pal for Sys {
    fn access(path: CStr, mode: c_int) -> Result<(), Errno> {
        let fd = File::open(path, fcntl::O_PATH | fcntl::O_CLOEXEC)?;

        if mode == F_OK {
            return Ok(());
        }

        let mut stat = syscall::Stat::default();

        e_raw(syscall::fstat(*fd as usize, &mut stat))?;

        let uid = e_raw(syscall::getuid())?;
        let gid = e_raw(syscall::getgid())?;

        let perms = if stat.st_uid as usize == uid {
            stat.st_mode >> (3 * 2 & 0o7)
        } else if stat.st_gid as usize == gid {
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

    fn brk(addr: *mut c_void) -> Result<*mut c_void, Errno> {
        unsafe {
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
                return Err(Errno(ENOMEM));
            }
        }
    }

    fn chdir(path: CStr) -> Result<(), Errno> {
        let path = path_from_c_str!(path)?;
        e_raw(path::chdir(path).map(|()| 0))?;
        Ok(())
    }

    fn chmod(path: CStr, mode: mode_t) -> Result<(), Errno> {
        let file = File::open(path, fcntl::O_PATH | fcntl::O_CLOEXEC)?;
        Self::fchmod(*file, mode)
    }

    fn chown(path: CStr, owner: uid_t, group: gid_t) -> Result<(), Errno> {
        let file = File::open(path, fcntl::O_PATH | fcntl::O_CLOEXEC)?;
        Self::fchown(*file, owner, group)
    }

    // FIXME: unsound
    fn clock_getres(clk_id: clockid_t, tp: *mut timespec) -> Result<(), Errno> {
        // TODO
        eprintln!("relibc clock_getres({}, {:p}): not implemented", clk_id, tp);
        Err(Errno(ENOSYS))
    }

    // FIXME: unsound
    fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> Result<(), Errno> {
        unsafe {
            e_raw(libredox::clock_gettime(clk_id as usize, tp).map(|()| 0))?;
        }
        Ok(())
    }

    // FIXME: unsound
    fn clock_settime(clk_id: clockid_t, tp: *const timespec) -> Result<(), Errno> {
        // TODO
        eprintln!(
            "relibc clock_settime({}, {:p}): not implemented",
            clk_id, tp
        );
        Err(Errno(ENOSYS))
    }

    fn close(fd: c_int) -> Result<(), Errno> {
        e_raw(syscall::close(fd as usize))?;
        Ok(())
    }

    fn dup(fd: c_int) -> Result<c_int, Errno> {
        e_raw(syscall::dup(fd as usize, &[])).map(|res| res as c_int)
    }

    fn dup2(fd1: c_int, fd2: c_int) -> Result<c_int, Errno> {
        e_raw(syscall::dup2(fd1 as usize, fd2 as usize, &[])).map(|res| res as c_int)
    }

    fn exit(status: c_int) -> ! {
        let _ = syscall::exit(status as usize);
        loop {}
    }

    unsafe fn execve(
        path: CStr,
        argv: *const *mut c_char,
        envp: *const *mut c_char,
    ) -> Result<(), Errno> {
        e_raw(self::exec::execve(
            Executable::AtPath(path),
            self::exec::ArgEnv::C { argv, envp },
            None,
        ))?;
        Ok(())
    }
    unsafe fn fexecve(
        fildes: c_int,
        argv: *const *mut c_char,
        envp: *const *mut c_char,
    ) -> Result<(), Errno> {
        e_raw(self::exec::execve(
            Executable::InFd {
                file: File::new(fildes),
                arg0: CStr::from_ptr(argv.read()).to_bytes(),
            },
            self::exec::ArgEnv::C { argv, envp },
            None,
        ))?;
        Ok(())
    }

    fn fchdir(fd: c_int) -> Result<(), Errno> {
        let mut buf = [0; 4096];
        let res = e_raw(syscall::fpath(fd as usize, &mut buf))?;
        match str::from_utf8(&buf[..res]) {
            Ok(path) => {
                e_raw(path::chdir(path).map(|()| 0))?;
                Ok(())
            }
            Err(_) => Err(Errno(EINVAL)),
        }
    }

    fn fchmod(fd: c_int, mode: mode_t) -> Result<(), Errno> {
        e_raw(syscall::fchmod(fd as usize, mode as u16))?;
        Ok(())
    }

    fn fchown(fd: c_int, owner: uid_t, group: gid_t) -> Result<(), Errno> {
        e_raw(syscall::fchown(fd as usize, owner as u32, group as u32))?;
        Ok(())
    }

    fn fcntl(fd: c_int, cmd: c_int, args: c_ulonglong) -> Result<c_int, Errno> {
        e_raw(syscall::fcntl(fd as usize, cmd as usize, args as usize)).map(|res| res as c_int)
    }

    fn fdatasync(fd: c_int) -> Result<(), Errno> {
        // TODO: "Needs" syscall update
        e_raw(syscall::fsync(fd as usize))?;
        Ok(())
    }

    fn flock(_fd: c_int, _operation: c_int) -> Result<(), Errno> {
        // TODO: Redox does not have file locking yet
        Ok(())
    }

    fn fork() -> Result<pid_t, Errno> {
        e_raw(clone::fork_impl()).map(|res| res as pid_t)
    }

    // FIXME: unsound
    fn fstat(fildes: c_int, buf: *mut stat) -> Result<(), Errno> {
        unsafe {
            e_raw(libredox::fstat(fildes as usize, buf).map(|()| 0))?;
        }
        Ok(())
    }

    // FIXME: unsound
    fn fstatvfs(fildes: c_int, buf: *mut statvfs) -> Result<(), Errno> {
        unsafe {
            e_raw(libredox::fstatvfs(fildes as usize, buf).map(|()| 0))?;
        }
        Ok(())
    }

    fn fsync(fd: c_int) -> Result<(), Errno> {
        e_raw(syscall::fsync(fd as usize))?;
        Ok(())
    }

    fn ftruncate(fd: c_int, len: off_t) -> Result<(), Errno> {
        e_raw(syscall::ftruncate(fd as usize, len as usize))?;
        Ok(())
    }

    // FIXME: unsound
    fn futex(addr: *mut c_int, op: c_int, val: c_int, val2: usize) -> Result<c_long, Errno> {
        match unsafe {
            syscall::futex(
                addr as *mut i32,
                op as usize,
                val as i32,
                val2,
                ptr::null_mut(),
            )
        } {
            Ok(success) => Ok(success as c_long),
            Err(err) => Err(Errno(err.errno)),
        }
    }

    // FIXME: unsound
    fn futimens(fd: c_int, times: *const timespec) -> Result<(), Errno> {
        unsafe {
            e_raw(libredox::futimens(fd as usize, times).map(|()| 0))?;
        }
        Ok(())
    }

    // FIXME: unsound
    fn utimens(path: CStr, times: *const timespec) -> Result<(), Errno> {
        let file = File::open(path, fcntl::O_PATH | fcntl::O_CLOEXEC)?;
        Self::futimens(*file, times)
    }

    // FIXME: unsound
    fn getcwd(buf: *mut c_char, size: size_t) -> Result<*mut c_char, Errno> {
        // TODO: Not using MaybeUninit seems a little unsafe

        let buf_slice = unsafe { slice::from_raw_parts_mut(buf as *mut u8, size as usize) };
        if buf_slice.is_empty() {
            return Err(Errno(EINVAL));
        }

        if path::getcwd(buf_slice).is_none() {
            return Err(Errno(ERANGE));
        }

        Ok(buf)
    }

    fn getdents(fd: c_int, mut dirents: *mut dirent, max_bytes: usize) -> Result<c_int, Errno> {
        //TODO: rewrite this code. Originally the *dirents = dirent { ... } stuff below caused
        // massive issues. This has been hacked around, but it still isn't perfect

        // Get initial reading position
        let mut read = e_raw(syscall::lseek(fd as usize, 0, syscall::SEEK_CUR))? as isize;

        let mut written = 0;
        let mut buf = [0; 1024];

        let mut name = [0; 256];
        let mut i = 0;

        let mut flush = |written: &mut usize, i: &mut usize, name: &mut [c_char; 256]| {
            if *i < name.len() {
                // Set NUL byte
                name[*i] = 0;
            }
            // Get size: full size - unused bytes
            if *written + mem::size_of::<dirent>() > max_bytes {
                // Seek back to after last read entry and return
                match syscall::lseek(fd as usize, read, syscall::SEEK_SET) {
                    Ok(_) => return Some(*written as c_int),
                    Err(err) => return Some(-err.errno),
                }
            }
            let size = mem::size_of::<dirent>() - name.len().saturating_sub(*i + 1);
            unsafe {
                //This is the offending code mentioned above
                *dirents = dirent {
                    d_ino: 0,
                    d_off: read as off_t,
                    d_reclen: size as c_ushort,
                    d_type: 0,
                    d_name: *name,
                };
                dirents = (dirents as *mut u8).offset(size as isize) as *mut dirent;
            }
            read += *i as isize + /* newline */ 1;
            *written += size;
            *i = 0;
            None
        };

        loop {
            // Read a chunk from the directory
            let len = match syscall::read(fd as usize, &mut buf) {
                Ok(0) => {
                    if i > 0 {
                        if let Some(value) = flush(&mut written, &mut i, &mut name) {
                            if value < 0 {
                                return Err(Errno(-value));
                            }
                            return Ok(value);
                        }
                    }
                    return Ok(written as c_int);
                }
                Ok(n) => n,
                Err(err) => return Err(Errno(err.errno)),
            };

            // Handle everything
            let mut start = 0;
            while start < len {
                let buf = &buf[start..len];

                // Copy everything up until a newline
                let newline = buf.iter().position(|&c| c == b'\n');
                let pre_len = newline.unwrap_or(buf.len());
                let post_len = newline.map(|i| i + 1).unwrap_or(buf.len());
                if i < pre_len {
                    // Reserve space for NUL byte
                    let name_len = name.len() - 1;
                    let name = &mut name[i..name_len];
                    let copy = pre_len.min(name.len());
                    let buf = unsafe { slice::from_raw_parts(buf.as_ptr() as *const c_char, copy) };
                    name[..copy].copy_from_slice(buf);
                }

                i += pre_len;
                start += post_len;

                // Write the directory entry
                if newline.is_some() {
                    if let Some(value) = flush(&mut written, &mut i, &mut name) {
                        if value < 0 {
                            return Err(Errno(-value));
                        }
                        return Ok(value);
                    }
                }
            }
        }
    }

    fn getegid() -> gid_t {
        syscall::getegid().unwrap() as gid_t
    }

    fn geteuid() -> uid_t {
        syscall::geteuid().unwrap() as uid_t
    }

    fn getgid() -> gid_t {
        syscall::getgid().unwrap() as gid_t
    }

    unsafe fn getgroups(size: c_int, list: *mut gid_t) -> Result<c_int, Errno> {
        // TODO
        eprintln!("relibc getgroups({}, {:p}): not implemented", size, list);
        Err(Errno(ENOSYS))
    }

    fn getpagesize() -> usize {
        PAGE_SIZE
    }

    fn getpgid(pid: pid_t) -> Result<pid_t, Errno> {
        e_raw(syscall::getpgid(pid as usize)).map(|res| res as pid_t)
    }

    fn getpid() -> pid_t {
        syscall::getpid().unwrap() as pid_t
    }

    fn getppid() -> pid_t {
        syscall::getppid().unwrap() as pid_t
    }

    fn getpriority(which: c_int, who: id_t) -> Result<c_int, Errno> {
        // TODO
        eprintln!("getpriority({}, {}): not implemented", which, who);
        Err(Errno(ENOSYS))
    }

    fn getrandom(buf: &mut [u8], flags: c_uint) -> Result<ssize_t, Errno> {
        //TODO: make this a system call?

        let path = if flags & sys_random::GRND_RANDOM != 0 {
            //TODO: /dev/random equivalent
            "rand:"
        } else {
            "rand:"
        };

        let mut open_flags = syscall::O_RDONLY | syscall::O_CLOEXEC;
        if flags & sys_random::GRND_NONBLOCK != 0 {
            open_flags |= syscall::O_NONBLOCK;
        }

        let fd = e_raw(syscall::open(path, open_flags))?;

        let res = e_raw(syscall::read(fd, buf)).map(|res| res as ssize_t);

        let _ = syscall::close(fd);

        res
    }

    unsafe fn getrlimit(resource: c_int, rlim: *mut rlimit) -> Result<(), Errno> {
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

    unsafe fn setrlimit(resource: c_int, rlim: *const rlimit) -> Result<(), Errno> {
        //TOOD
        eprintln!(
            "relibc setrlimit({}, {:p}): not implemented",
            resource, rlim
        );
        Err(Errno(EPERM))
    }

    fn getsid(pid: pid_t) -> Result<pid_t, Errno> {
        //TODO
        eprintln!("relibc getsid({}): not implemented", pid);
        Err(Errno(ENOSYS))
    }

    fn gettid() -> pid_t {
        //TODO
        Self::getpid()
    }

    fn gettimeofday(tp: *mut timeval, tzp: *mut timezone) -> Result<(), Errno> {
        let mut redox_tp = redox_timespec::default();
        e_raw(syscall::clock_gettime(
            syscall::CLOCK_REALTIME,
            &mut redox_tp,
        ))?;
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
        syscall::getuid().unwrap() as pid_t
    }

    fn lchown(path: CStr, owner: uid_t, group: gid_t) -> Result<(), Errno> {
        // TODO: Is it correct for regular chown to use O_PATH? On Linux the meaning of that flag
        // is to forbid file operations, including fchown.

        // unlike chown, never follow symbolic links
        let file = File::open(path, fcntl::O_CLOEXEC | fcntl::O_NOFOLLOW)?;
        Self::fchown(*file, owner, group)
    }

    fn link(path1: CStr, path2: CStr) -> Result<(), Errno> {
        e_raw(unsafe { syscall::link(path1.as_ptr() as *const u8, path2.as_ptr() as *const u8) })?;
        Ok(())
    }

    fn lseek(fd: c_int, offset: off_t, whence: c_int) -> Result<off_t, Errno> {
        e_raw(syscall::lseek(
            fd as usize,
            offset as isize,
            whence as usize,
        ))
        .map(|res| res as off_t)
    }

    fn mkdir(path: CStr, mode: mode_t) -> Result<(), Errno> {
        File::create(
            path,
            fcntl::O_DIRECTORY | fcntl::O_EXCL | fcntl::O_CLOEXEC,
            0o777,
        )?;
        Ok(())
    }

    fn mkfifo(path: CStr, mode: mode_t) -> Result<(), Errno> {
        File::create(
            path,
            fcntl::O_CREAT | fcntl::O_CLOEXEC,
            syscall::MODE_FIFO as mode_t | (mode & 0o777),
        )?;
        Ok(())
    }

    unsafe fn mlock(addr: *const c_void, len: usize) -> Result<(), Errno> {
        // Redox never swaps
        Ok(())
    }

    fn mlockall(flags: c_int) -> Result<(), Errno> {
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
    ) -> Result<*mut c_void, Errno> {
        let map = Map {
            offset: off as usize,
            size: round_up_to_page_size(len),
            flags: syscall::MapFlags::from_bits_truncate(
                ((prot as usize) << 16) | ((flags as usize) & 0xFFFF),
            ),
            address: addr as usize,
        };

        if flags & MAP_ANONYMOUS == MAP_ANONYMOUS {
            e_raw(syscall::fmap(!0, &map)).map(|res| res as *mut c_void)
        } else {
            e_raw(syscall::fmap(fildes as usize, &map)).map(|res| res as *mut c_void)
        }
    }

    unsafe fn mprotect(addr: *mut c_void, len: usize, prot: c_int) -> Result<(), Errno> {
        e_raw(syscall::mprotect(
            addr as usize,
            round_up_to_page_size(len),
            syscall::MapFlags::from_bits((prot as usize) << 16)
                .expect("mprotect: invalid bit pattern"),
        ))?;
        Ok(())
    }

    unsafe fn msync(addr: *mut c_void, len: usize, flags: c_int) -> Result<(), Errno> {
        eprintln!(
            "relibc msync({:p}, 0x{:x}, 0x{:x}): not implemented",
            addr, len, flags
        );
        e_raw(Err(syscall::Error::new(syscall::ENOSYS)))?;
        Ok(())
        /* TODO
        e_raw(syscall::msync(
            addr as usize,
            round_up_to_page_size(len),
            flags
        ))?;
        Ok(())
        */
    }

    unsafe fn munlock(addr: *const c_void, len: usize) -> Result<(), Errno> {
        // Redox never swaps
        Ok(())
    }

    fn munlockall() -> Result<(), Errno> {
        // Redox never swaps
        Ok(())
    }

    unsafe fn munmap(addr: *mut c_void, len: usize) -> Result<(), Errno> {
        e_raw(syscall::funmap(addr as usize, round_up_to_page_size(len)))?;
        Ok(())
    }

    unsafe fn madvise(addr: *mut c_void, len: usize, flags: c_int) -> Result<(), Errno> {
        eprintln!(
            "relibc madvise({:p}, 0x{:x}, 0x{:x}): not implemented",
            addr, len, flags
        );
        e_raw(Err(syscall::Error::new(syscall::ENOSYS)))?;
        Ok(())
    }

    fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> Result<(), Errno> {
        let redox_rqtp = unsafe { redox_timespec::from(&*rqtp) };
        let mut redox_rmtp: redox_timespec;
        if rmtp.is_null() {
            redox_rmtp = redox_timespec::default();
        } else {
            redox_rmtp = unsafe { redox_timespec::from(&*rmtp) };
        }
        e_raw(syscall::nanosleep(&redox_rqtp, &mut redox_rmtp))?;
        unsafe {
            if !rmtp.is_null() {
                (*rmtp).tv_sec = redox_rmtp.tv_sec as time_t;
                (*rmtp).tv_nsec = redox_rmtp.tv_nsec as c_long;
            }
        }
        Ok(())
    }

    fn open(path: CStr, oflag: c_int, mode: mode_t) -> Result<c_int, Errno> {
        let path = path_from_c_str!(path)?;

        e_raw(libredox::open(path, oflag, mode)).map(|res| res as c_int)
    }

    fn pipe2(fds: &mut [c_int], flags: c_int) -> Result<(), Errno> {
        extra::pipe2(fds, flags as usize).map_err(|err| Errno(err.errno as c_int))
    }

    unsafe fn rlct_clone(stack: *mut usize) -> Result<crate::pthread::OsTid, Errno> {
        clone::rlct_clone_impl(stack)
            .map(|context_id| crate::pthread::OsTid { context_id })
            .map_err(|error| Errno(error.errno))
    }
    unsafe fn rlct_kill(os_tid: crate::pthread::OsTid, signal: usize) -> Result<(), Errno> {
        syscall::kill(os_tid.context_id, signal).map_err(|error| Errno(error.errno))?;
        Ok(())
    }
    fn current_os_tid() -> crate::pthread::OsTid {
        // TODO
        crate::pthread::OsTid {
            context_id: Self::getpid() as _,
        }
    }

    fn read(fd: c_int, buf: &mut [u8]) -> Result<ssize_t, Errno> {
        e_raw(syscall::read(fd as usize, buf)).map(|res| res as ssize_t)
    }

    fn fpath(fildes: c_int, out: &mut [u8]) -> Result<ssize_t, Errno> {
        e_raw(syscall::fpath(fildes as usize, out)).map(|res| res as ssize_t)
    }

    fn readlink(pathname: CStr, out: &mut [u8]) -> Result<ssize_t, Errno> {
        let file = File::open(
            pathname,
            fcntl::O_RDONLY | fcntl::O_SYMLINK | fcntl::O_CLOEXEC,
        )?;
        Self::read(*file, out)
    }

    fn rename(oldpath: CStr, newpath: CStr) -> Result<(), Errno> {
        let newpath = path_from_c_str!(newpath)?;
        let file = File::open(oldpath, fcntl::O_PATH | fcntl::O_CLOEXEC)?;
        e_raw(syscall::frename(*file as usize, newpath))?;
        Ok(())
    }

    fn rmdir(path: CStr) -> Result<(), Errno> {
        let path = path_from_c_str!(path)?;
        e_raw(canonicalize(path).and_then(|path| syscall::rmdir(&path)))?;
        Ok(())
    }

    fn sched_yield() -> Result<(), Errno> {
        e_raw(syscall::sched_yield())?;
        Ok(())
    }

    unsafe fn setgroups(size: size_t, list: *const gid_t) -> Result<(), Errno> {
        // TODO
        eprintln!("relibc setgroups({}, {:p}): not implemented", size, list);
        Err(Errno(ENOSYS))
    }

    fn setpgid(pid: pid_t, pgid: pid_t) -> Result<(), Errno> {
        e_raw(syscall::setpgid(pid as usize, pgid as usize))?;
        Ok(())
    }

    fn setpriority(which: c_int, who: id_t, prio: c_int) -> Result<(), Errno> {
        // TODO
        eprintln!(
            "relibc setpriority({}, {}, {}): not implemented",
            which, who, prio
        );
        Err(Errno(ENOSYS))
    }

    fn setsid() -> Result<(), Errno> {
        // TODO
        eprintln!("relibc setsid(): not implemented");
        Err(Errno(ENOSYS))
    }

    fn setregid(rgid: gid_t, egid: gid_t) -> Result<(), Errno> {
        e_raw(syscall::setregid(rgid as usize, egid as usize))?;
        Ok(())
    }

    fn setreuid(ruid: uid_t, euid: uid_t) -> Result<(), Errno> {
        e_raw(syscall::setreuid(ruid as usize, euid as usize))?;
        Ok(())
    }

    fn symlink(path1: CStr, path2: CStr) -> Result<(), Errno> {
        let mut file = File::create(
            path2,
            fcntl::O_WRONLY | fcntl::O_SYMLINK | fcntl::O_CLOEXEC,
            0o777,
        )?;

        file.write(path1.to_bytes())?;

        Ok(())
    }

    fn sync() -> Result<(), Errno> {
        Ok(())
    }

    fn umask(mask: mode_t) -> mode_t {
        e_raw(syscall::umask(mask as usize)).unwrap() as mode_t
    }

    fn uname(utsname: *mut utsname) -> Result<(), Errno> {
        fn gethostname(name: &mut [u8]) -> io::Result<()> {
            if name.is_empty() {
                return Ok(());
            }

            let mut file = File::open(c_str!("/etc/hostname"), fcntl::O_RDONLY | fcntl::O_CLOEXEC)?;

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

        fn inner(utsname: *mut utsname) -> CoreResult<(), i32> {
            match gethostname(unsafe {
                slice::from_raw_parts_mut(
                    (*utsname).nodename.as_mut_ptr() as *mut u8,
                    (*utsname).nodename.len(),
                )
            }) {
                Ok(_) => (),
                Err(_) => return Err(EIO),
            }

            let file_path = c_str!("sys:uname");
            let mut file = match File::open(file_path, fcntl::O_RDONLY | fcntl::O_CLOEXEC) {
                Ok(ok) => ok,
                Err(_) => return Err(EIO),
            };
            let mut lines = BufReader::new(&mut file).lines();

            let mut read_line = |dst: &mut [c_char]| {
                let line = match lines.next() {
                    Some(Ok(l)) => match CString::new(l) {
                        Ok(l) => l,
                        Err(_) => return Err(EIO),
                    },
                    None | Some(Err(_)) => return Err(EIO),
                };

                let line_slice: &[c_char] = unsafe { mem::transmute(line.as_bytes_with_nul()) };

                if line_slice.len() <= UTSLENGTH {
                    dst[..line_slice.len()].copy_from_slice(line_slice);
                    Ok(())
                } else {
                    Err(EIO)
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

        inner(utsname).map_err(|err| Errno(err))
    }

    fn unlink(path: CStr) -> Result<(), Errno> {
        let path = path_from_c_str!(path)?;
        e_raw(canonicalize(path).and_then(|path| syscall::unlink(&path)))?;
        Ok(())
    }

    fn waitpid(mut pid: pid_t, stat_loc: *mut c_int, options: c_int) -> Result<pid_t, Errno> {
        if pid == !0 {
            pid = 0;
        }
        let mut res = None;
        let mut status = 0;

        let inner = |status: &mut usize, flags| {
            syscall::waitpid(
                pid as usize,
                status,
                syscall::WaitFlags::from_bits(flags as usize)
                    .expect("waitpid: invalid bit pattern"),
            )
        };

        // First, allow ptrace to handle waitpid
        // TODO: Handle special PIDs here (such as -1)
        let state = ptrace::init_state();
        let mut sessions = state.sessions.lock();
        if let Ok(session) = ptrace::get_session(&mut sessions, pid) {
            if options & sys_wait::WNOHANG != sys_wait::WNOHANG {
                let mut _event = PtraceEvent::default();
                let _ = (&mut &session.tracer).read(&mut _event);

                res = Some(e_raw(inner(
                    &mut status,
                    options | sys_wait::WNOHANG | sys_wait::WUNTRACED,
                )));
                if res == Some(Ok(0)) {
                    // WNOHANG, just pretend ptrace SIGSTOP:ped this
                    status = (syscall::SIGSTOP << 8) | 0x7f;
                    assert!(syscall::wifstopped(status));
                    assert_eq!(syscall::wstopsig(status), syscall::SIGSTOP);
                    res = Some(Ok(pid as usize));
                }
            }
        }

        // If ptrace didn't impact this waitpid, proceed *almost* as
        // normal: We still need to add WUNTRACED, but we only return
        // it if (and only if) a ptrace traceme was activated during
        // the wait.
        let res = res
            .unwrap_or_else(|| loop {
                let res = e_raw(inner(&mut status, options | sys_wait::WUNTRACED));

                // TODO: Also handle special PIDs here
                if !syscall::wifstopped(status) || ptrace::is_traceme(pid) {
                    break res;
                }
            })
            .map(|res| res as pid_t);

        // If stat_loc is non-null, set that and the return
        unsafe {
            if !stat_loc.is_null() {
                *stat_loc = status as c_int;
            }
        }
        res
    }

    fn write(fd: c_int, buf: &[u8]) -> Result<ssize_t, Errno> {
        e_raw(syscall::write(fd as usize, buf)).map(|res| res as ssize_t)
    }

    fn verify() -> Result<(), Errno> {
        // GETPID on Redox is 20, which is WRITEV on Linux
        e_raw(unsafe { syscall::syscall5(syscall::number::SYS_GETPID, !0, !0, !0, !0, !0) })?;
        Ok(())
    }

    fn exit_thread() -> ! {
        Self::exit(0)
    }
}
