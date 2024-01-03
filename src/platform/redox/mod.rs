use core::{convert::TryFrom, mem, ptr, result::Result as CoreResult, slice, str};
use syscall::{
    self,
    data::{Map, Stat as redox_stat, StatVfs as redox_statvfs, TimeSpec as redox_timespec},
    Error, PtraceEvent, Result, EMFILE,
};

use crate::{
    c_str::{CStr, CString},
    fs::File,
    header::{
        dirent::dirent,
        errno::{EBADR, EINVAL, EIO, ENOENT, ENOMEM, ENOSYS, EPERM, ERANGE},
        fcntl,
        sys_mman::{MAP_ANONYMOUS, MAP_FAILED, PROT_READ, PROT_WRITE},
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
mod libcscheme;
mod libredox;
pub(crate) mod path;
mod ptrace;
mod signal;
mod socket;

macro_rules! path_from_c_str {
    ($c_str:expr) => {{
        match $c_str.to_str() {
            Ok(ok) => ok,
            Err(err) => {
                unsafe {
                    errno = EINVAL;
                }
                return -1;
            }
        }
    }};
}

use self::{exec::Executable, path::canonicalize};

pub fn e(sys: Result<usize>) -> usize {
    match sys {
        Ok(ok) => ok,
        Err(err) => {
            unsafe {
                errno = err.errno as c_int;
            }
            !0
        }
    }
}

pub struct Sys;

impl Pal for Sys {
    fn access(path: CStr, mode: c_int) -> c_int {
        let fd = match File::open(path, fcntl::O_PATH | fcntl::O_CLOEXEC) {
            Ok(fd) => fd,
            Err(_) => return -1,
        };

        if mode == F_OK {
            return 0;
        }

        let mut stat = syscall::Stat::default();

        if e(syscall::fstat(*fd as usize, &mut stat)) == !0 {
            return -1;
        }

        let uid = e(syscall::getuid());
        if uid == !0 {
            return -1;
        }
        let gid = e(syscall::getgid());
        if gid == !0 {
            return -1;
        }

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
            unsafe {
                errno = EINVAL;
            }
            return -1;
        }

        0
    }

    fn brk(addr: *mut c_void) -> *mut c_void {
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
                );
                if allocated == !0 as *mut c_void
                /* MAP_FAILED */
                {
                    return !0 as *mut c_void;
                }

                BRK_CUR = allocated;
                BRK_END = (allocated as *mut u8).add(BRK_MAX_SIZE) as *mut c_void;
            }

            if addr.is_null() {
                // Lookup what previous brk() invocations have set the address to
                BRK_CUR
            } else if BRK_CUR <= addr && addr < BRK_END {
                // It's inside buffer, return
                BRK_CUR = addr;
                addr
            } else {
                // It was outside of valid range
                errno = ENOMEM;
                ptr::null_mut()
            }
        }
    }

    fn chdir(path: CStr) -> c_int {
        let path = path_from_c_str!(path);
        e(path::chdir(path).map(|()| 0)) as c_int
    }

    fn chmod(path: CStr, mode: mode_t) -> c_int {
        match File::open(path, fcntl::O_PATH | fcntl::O_CLOEXEC) {
            Ok(file) => Self::fchmod(*file, mode),
            Err(_) => -1,
        }
    }

    fn chown(path: CStr, owner: uid_t, group: gid_t) -> c_int {
        match File::open(path, fcntl::O_PATH | fcntl::O_CLOEXEC) {
            Ok(file) => Self::fchown(*file, owner, group),
            Err(_) => -1,
        }
    }

    // FIXME: unsound
    fn clock_getres(clk_id: clockid_t, tp: *mut timespec) -> c_int {
        // TODO
        eprintln!("relibc clock_getres({}, {:p}): not implemented", clk_id, tp);
        unsafe { errno = ENOSYS }
        -1
    }

    // FIXME: unsound
    fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> c_int {
        unsafe { e(libredox::clock_gettime(clk_id as usize, tp).map(|()| 0)) as c_int }
    }

    // FIXME: unsound
    fn clock_settime(clk_id: clockid_t, tp: *const timespec) -> c_int {
        // TODO
        eprintln!(
            "relibc clock_settime({}, {:p}): not implemented",
            clk_id, tp
        );
        unsafe { errno = ENOSYS };
        -1
    }

    fn close(fd: c_int) -> c_int {
        e(syscall::close(fd as usize)) as c_int
    }

    fn dup(fd: c_int) -> c_int {
        e(syscall::dup(fd as usize, &[])) as c_int
    }

    fn dup2(fd1: c_int, fd2: c_int) -> c_int {
        e(syscall::dup2(fd1 as usize, fd2 as usize, &[])) as c_int
    }

    fn exit(status: c_int) -> ! {
        let _ = syscall::exit(status as usize);
        loop {}
    }

    unsafe fn execve(path: CStr, argv: *const *mut c_char, envp: *const *mut c_char) -> c_int {
        e(self::exec::execve(
            Executable::AtPath(path),
            self::exec::ArgEnv::C { argv, envp },
            None,
        )) as c_int
    }
    unsafe fn fexecve(fildes: c_int, argv: *const *mut c_char, envp: *const *mut c_char) -> c_int {
        e(self::exec::execve(
            Executable::InFd {
                file: File::new(fildes),
                arg0: CStr::from_ptr(argv.read()).to_bytes(),
            },
            self::exec::ArgEnv::C { argv, envp },
            None,
        )) as c_int
    }

    fn fchdir(fd: c_int) -> c_int {
        let mut buf = [0; 4096];
        let res = e(syscall::fpath(fd as usize, &mut buf));
        if res == !0 {
            !0
        } else {
            match str::from_utf8(&buf[..res]) {
                Ok(path) => e(path::chdir(path).map(|()| 0)) as c_int,
                Err(_) => {
                    unsafe { errno = EINVAL };
                    return -1;
                }
            }
        }
    }

    fn fchmod(fd: c_int, mode: mode_t) -> c_int {
        e(syscall::fchmod(fd as usize, mode as u16)) as c_int
    }

    fn fchown(fd: c_int, owner: uid_t, group: gid_t) -> c_int {
        e(syscall::fchown(fd as usize, owner as u32, group as u32)) as c_int
    }

    fn fcntl(fd: c_int, cmd: c_int, args: c_ulonglong) -> c_int {
        e(syscall::fcntl(fd as usize, cmd as usize, args as usize)) as c_int
    }

    fn fdatasync(fd: c_int) -> c_int {
        // TODO: "Needs" syscall update
        e(syscall::fsync(fd as usize)) as c_int
    }

    fn flock(_fd: c_int, _operation: c_int) -> c_int {
        // TODO: Redox does not have file locking yet
        0
    }

    fn fork() -> pid_t {
        e(clone::fork_impl()) as pid_t
    }

    // FIXME: unsound
    fn fstat(fildes: c_int, buf: *mut stat) -> c_int {
        unsafe { e(libredox::fstat(fildes as usize, buf).map(|()| 0)) as c_int }
    }

    // FIXME: unsound
    fn fstatvfs(fildes: c_int, buf: *mut statvfs) -> c_int {
        unsafe { e(libredox::fstatvfs(fildes as usize, buf).map(|()| 0)) as c_int }
    }

    fn fsync(fd: c_int) -> c_int {
        e(syscall::fsync(fd as usize)) as c_int
    }

    fn ftruncate(fd: c_int, len: off_t) -> c_int {
        e(syscall::ftruncate(fd as usize, len as usize)) as c_int
    }

    // FIXME: unsound
    fn futex(
        addr: *mut c_int,
        op: c_int,
        val: c_int,
        val2: usize,
    ) -> Result<c_long, crate::pthread::Errno> {
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
            Err(err) => Err(crate::pthread::Errno(err.errno)),
        }
    }

    // FIXME: unsound
    fn futimens(fd: c_int, times: *const timespec) -> c_int {
        unsafe { e(libredox::futimens(fd as usize, times).map(|()| 0)) as c_int }
    }

    // FIXME: unsound
    fn utimens(path: CStr, times: *const timespec) -> c_int {
        match File::open(path, fcntl::O_PATH | fcntl::O_CLOEXEC) {
            Ok(file) => Self::futimens(*file, times),
            Err(_) => -1,
        }
    }

    // FIXME: unsound
    fn getcwd(buf: *mut c_char, size: size_t) -> *mut c_char {
        // TODO: Not using MaybeUninit seems a little unsafe

        let buf_slice = unsafe { slice::from_raw_parts_mut(buf as *mut u8, size as usize) };
        if buf_slice.is_empty() {
            unsafe {
                errno = EINVAL;
            }
            return ptr::null_mut();
        }

        if path::getcwd(buf_slice).is_none() {
            unsafe {
                errno = ERANGE;
            }
            return ptr::null_mut();
        }

        buf
    }

    fn getdents(fd: c_int, mut dirents: *mut dirent, max_bytes: usize) -> c_int {
        //TODO: rewrite this code. Originally the *dirents = dirent { ... } stuff below caused
        // massive issues. This has been hacked around, but it still isn't perfect

        // Get initial reading position
        let mut read = match syscall::lseek(fd as usize, 0, syscall::SEEK_CUR) {
            Ok(pos) => pos as isize,
            Err(err) => return -err.errno,
        };

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
                            return value;
                        }
                    }
                    return written as c_int;
                }
                Ok(n) => n,
                Err(err) => return -err.errno,
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
                        return value;
                    }
                }
            }
        }
    }

    fn getegid() -> gid_t {
        e(syscall::getegid()) as gid_t
    }

    fn geteuid() -> uid_t {
        e(syscall::geteuid()) as uid_t
    }

    fn getgid() -> gid_t {
        e(syscall::getgid()) as gid_t
    }

    unsafe fn getgroups(size: c_int, list: *mut gid_t) -> c_int {
        // TODO
        eprintln!("relibc getgroups({}, {:p}): not implemented", size, list);
        unsafe { errno = ENOSYS };
        -1
    }

    fn getpagesize() -> usize {
        PAGE_SIZE
    }

    fn getpgid(pid: pid_t) -> pid_t {
        e(syscall::getpgid(pid as usize)) as pid_t
    }

    fn getpid() -> pid_t {
        e(syscall::getpid()) as pid_t
    }

    fn getppid() -> pid_t {
        e(syscall::getppid()) as pid_t
    }

    fn getpriority(which: c_int, who: id_t) -> c_int {
        // TODO
        eprintln!("getpriority({}, {}): not implemented", which, who);
        unsafe { errno = ENOSYS };
        -1
    }

    fn getrandom(buf: &mut [u8], flags: c_uint) -> ssize_t {
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

        let fd = e(syscall::open(path, open_flags));
        if fd == !0 {
            return -1;
        }

        let res = e(syscall::read(fd, buf)) as ssize_t;

        let _ = syscall::close(fd);

        res
    }

    unsafe fn getrlimit(resource: c_int, rlim: *mut rlimit) -> c_int {
        //TODO
        eprintln!(
            "relibc getrlimit({}, {:p}): not implemented",
            resource, rlim
        );
        if !rlim.is_null() {
            (*rlim).rlim_cur = RLIM_INFINITY;
            (*rlim).rlim_max = RLIM_INFINITY;
        }
        0
    }

    unsafe fn setrlimit(resource: c_int, rlim: *const rlimit) -> c_int {
        //TOOD
        eprintln!(
            "relibc setrlimit({}, {:p}): not implemented",
            resource, rlim
        );
        unsafe { errno = EPERM };
        -1
    }

    fn getsid(pid: pid_t) -> pid_t {
        //TODO
        eprintln!("relibc getsid({}): not implemented", pid);
        unsafe { errno = ENOSYS };
        -1
    }

    fn gettid() -> pid_t {
        //TODO
        Self::getpid()
    }

    fn gettimeofday(tp: *mut timeval, tzp: *mut timezone) -> c_int {
        let mut redox_tp = redox_timespec::default();
        let err = e(syscall::clock_gettime(
            syscall::CLOCK_REALTIME,
            &mut redox_tp,
        )) as c_int;
        if err < 0 {
            return err;
        }
        unsafe {
            (*tp).tv_sec = redox_tp.tv_sec as time_t;
            (*tp).tv_usec = (redox_tp.tv_nsec / 1000) as suseconds_t;

            if !tzp.is_null() {
                (*tzp).tz_minuteswest = 0;
                (*tzp).tz_dsttime = 0;
            }
        }
        0
    }

    fn getuid() -> uid_t {
        e(syscall::getuid()) as pid_t
    }

    fn lchown(path: CStr, owner: uid_t, group: gid_t) -> c_int {
        // TODO: Is it correct for regular chown to use O_PATH? On Linux the meaning of that flag
        // is to forbid file operations, including fchown.

        // unlike chown, never follow symbolic links
        match File::open(path, fcntl::O_CLOEXEC | fcntl::O_NOFOLLOW) {
            Ok(file) => Self::fchown(*file, owner, group),
            Err(_) => -1,
        }
    }

    fn link(path1: CStr, path2: CStr) -> c_int {
        e(unsafe { syscall::link(path1.as_ptr() as *const u8, path2.as_ptr() as *const u8) })
            as c_int
    }

    fn lseek(fd: c_int, offset: off_t, whence: c_int) -> off_t {
        e(syscall::lseek(
            fd as usize,
            offset as isize,
            whence as usize,
        )) as off_t
    }

    fn mkdir(path: CStr, mode: mode_t) -> c_int {
        match File::create(
            path,
            fcntl::O_DIRECTORY | fcntl::O_EXCL | fcntl::O_CLOEXEC,
            0o777,
        ) {
            Ok(_fd) => 0,
            Err(_) => -1,
        }
    }

    fn mkfifo(path: CStr, mode: mode_t) -> c_int {
        Sys::mknod(path, syscall::MODE_FIFO as mode_t | (mode & 0o777), 0)
    }

    fn mknodat(dir_fd: c_int, path_name: CStr, mode: mode_t, dev: dev_t) -> c_int {
        let mut dir_path_buf = [0; 4096];
        let res = Sys::fpath(dir_fd, &mut dir_path_buf);
        if res < 0 {
            return !0;
        }

        let dir_path = match str::from_utf8(&dir_path_buf[..res as usize]) {
            Ok(path) => path,
            Err(_) => {
                unsafe { errno = EBADR };
                return !0;
            }
        };

        let resource_path =
            match path::canonicalize_using_cwd(Some(&dir_path), &path_name.to_string_lossy()) {
                Some(path) => path,
                None => {
                    // Since parent_dir_path is resolved by fpath, it is more likely that
                    // the problem was with path.
                    unsafe { errno = ENOENT };
                    return !0;
                }
            };

        Sys::mknod(
            CStr::borrow(&CString::new(resource_path.as_bytes()).unwrap()),
            mode,
            dev,
        )
    }

    fn mknod(path: CStr, mode: mode_t, dev: dev_t) -> c_int {
        match File::create(path, fcntl::O_CREAT | fcntl::O_CLOEXEC, mode) {
            Ok(fd) => 0,
            Err(_) => -1,
        }
    }

    unsafe fn mlock(addr: *const c_void, len: usize) -> c_int {
        // Redox never swaps
        0
    }

    fn mlockall(flags: c_int) -> c_int {
        // Redox never swaps
        0
    }

    unsafe fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: c_int,
        flags: c_int,
        fildes: c_int,
        off: off_t,
    ) -> *mut c_void {
        let map = Map {
            offset: off as usize,
            size: round_up_to_page_size(len),
            flags: syscall::MapFlags::from_bits_truncate(
                ((prot as usize) << 16) | ((flags as usize) & 0xFFFF),
            ),
            address: addr as usize,
        };

        if flags & MAP_ANONYMOUS == MAP_ANONYMOUS {
            e(syscall::fmap(!0, &map)) as *mut c_void
        } else {
            e(syscall::fmap(fildes as usize, &map)) as *mut c_void
        }
    }

    unsafe fn mremap(
        addr: *mut c_void,
        len: usize,
        new_len: usize,
        flags: c_int,
        args: *mut c_void,
    ) -> *mut c_void {
        MAP_FAILED
    }

    unsafe fn mprotect(addr: *mut c_void, len: usize, prot: c_int) -> c_int {
        e(syscall::mprotect(
            addr as usize,
            round_up_to_page_size(len),
            syscall::MapFlags::from_bits((prot as usize) << 16)
                .expect("mprotect: invalid bit pattern"),
        )) as c_int
    }

    unsafe fn msync(addr: *mut c_void, len: usize, flags: c_int) -> c_int {
        eprintln!(
            "relibc msync({:p}, 0x{:x}, 0x{:x}): not implemented",
            addr, len, flags
        );
        e(Err(syscall::Error::new(syscall::ENOSYS))) as c_int
        /* TODO
        e(syscall::msync(
            addr as usize,
            round_up_to_page_size(len),
            flags
        )) as c_int
        */
    }

    unsafe fn munlock(addr: *const c_void, len: usize) -> c_int {
        // Redox never swaps
        0
    }

    fn munlockall() -> c_int {
        // Redox never swaps
        0
    }

    unsafe fn munmap(addr: *mut c_void, len: usize) -> c_int {
        if e(syscall::funmap(addr as usize, round_up_to_page_size(len))) == !0 {
            return !0;
        }
        0
    }

    unsafe fn madvise(addr: *mut c_void, len: usize, flags: c_int) -> c_int {
        eprintln!(
            "relibc madvise({:p}, 0x{:x}, 0x{:x}): not implemented",
            addr, len, flags
        );
        e(Err(syscall::Error::new(syscall::ENOSYS))) as c_int
    }

    fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> c_int {
        let redox_rqtp = unsafe { redox_timespec::from(&*rqtp) };
        let mut redox_rmtp: redox_timespec;
        if rmtp.is_null() {
            redox_rmtp = redox_timespec::default();
        } else {
            redox_rmtp = unsafe { redox_timespec::from(&*rmtp) };
        }
        match e(syscall::nanosleep(&redox_rqtp, &mut redox_rmtp)) as c_int {
            -1 => -1,
            _ => {
                unsafe {
                    if !rmtp.is_null() {
                        (*rmtp).tv_sec = redox_rmtp.tv_sec as time_t;
                        (*rmtp).tv_nsec = redox_rmtp.tv_nsec as c_long;
                    }
                }
                0
            }
        }
    }

    fn open(path: CStr, oflag: c_int, mode: mode_t) -> c_int {
        let path = path_from_c_str!(path);

        if path.starts_with(libcscheme::LIBC_SCHEME) {
            return e(libcscheme::open(path, oflag, mode)) as c_int;
        }

        e(libredox::open(path, oflag, mode)) as c_int
    }

    fn pipe2(fds: &mut [c_int], flags: c_int) -> c_int {
        e(extra::pipe2(fds, flags as usize).map(|()| 0)) as c_int
    }

    unsafe fn rlct_clone(
        stack: *mut usize,
    ) -> Result<crate::pthread::OsTid, crate::pthread::Errno> {
        clone::rlct_clone_impl(stack)
            .map(|context_id| crate::pthread::OsTid { context_id })
            .map_err(|error| crate::pthread::Errno(error.errno))
    }
    unsafe fn rlct_kill(
        os_tid: crate::pthread::OsTid,
        signal: usize,
    ) -> Result<(), crate::pthread::Errno> {
        syscall::kill(os_tid.context_id, signal)
            .map_err(|error| crate::pthread::Errno(error.errno))?;
        Ok(())
    }
    fn current_os_tid() -> crate::pthread::OsTid {
        // TODO
        crate::pthread::OsTid {
            context_id: Self::getpid() as _,
        }
    }

    fn read(fd: c_int, buf: &mut [u8]) -> ssize_t {
        e(syscall::read(fd as usize, buf)) as ssize_t
    }

    fn fpath(fildes: c_int, out: &mut [u8]) -> ssize_t {
        e(syscall::fpath(fildes as usize, out)) as ssize_t
    }

    fn readlink(pathname: CStr, out: &mut [u8]) -> ssize_t {
        match File::open(
            pathname,
            fcntl::O_RDONLY | fcntl::O_SYMLINK | fcntl::O_CLOEXEC,
        ) {
            Ok(file) => Self::read(*file, out),
            Err(_) => return -1,
        }
    }

    fn rename(oldpath: CStr, newpath: CStr) -> c_int {
        let newpath = path_from_c_str!(newpath);
        match File::open(oldpath, fcntl::O_PATH | fcntl::O_CLOEXEC) {
            Ok(file) => e(syscall::frename(*file as usize, newpath)) as c_int,
            Err(_) => -1,
        }
    }

    fn rmdir(path: CStr) -> c_int {
        let path = path_from_c_str!(path);
        e(canonicalize(path).and_then(|path| syscall::rmdir(&path))) as c_int
    }

    fn sched_yield() -> c_int {
        e(syscall::sched_yield()) as c_int
    }

    unsafe fn setgroups(size: size_t, list: *const gid_t) -> c_int {
        // TODO
        eprintln!("relibc setgroups({}, {:p}): not implemented", size, list);
        unsafe { errno = ENOSYS };
        -1
    }

    fn setpgid(pid: pid_t, pgid: pid_t) -> c_int {
        e(syscall::setpgid(pid as usize, pgid as usize)) as c_int
    }

    fn setpriority(which: c_int, who: id_t, prio: c_int) -> c_int {
        // TODO
        eprintln!(
            "relibc setpriority({}, {}, {}): not implemented",
            which, who, prio
        );
        unsafe { errno = ENOSYS };
        -1
    }

    fn setsid() -> c_int {
        // TODO
        eprintln!("relibc setsid(): not implemented");
        unsafe { errno = ENOSYS };
        -1
    }

    fn setregid(rgid: gid_t, egid: gid_t) -> c_int {
        e(syscall::setregid(rgid as usize, egid as usize)) as c_int
    }

    fn setreuid(ruid: uid_t, euid: uid_t) -> c_int {
        e(syscall::setreuid(ruid as usize, euid as usize)) as c_int
    }

    fn symlink(path1: CStr, path2: CStr) -> c_int {
        let mut file = match File::create(
            path2,
            fcntl::O_WRONLY | fcntl::O_SYMLINK | fcntl::O_CLOEXEC,
            0o777,
        ) {
            Ok(ok) => ok,
            Err(_) => return -1,
        };

        if file.write(path1.to_bytes()).is_err() {
            return -1;
        }

        0
    }

    fn sync() -> c_int {
        0
    }

    fn umask(mask: mode_t) -> mode_t {
        e(syscall::umask(mask as usize)) as mode_t
    }

    fn uname(utsname: *mut utsname) -> c_int {
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

        match inner(utsname) {
            Ok(()) => 0,
            Err(err) => unsafe {
                errno = err;
                -1
            },
        }
    }

    fn unlink(path: CStr) -> c_int {
        let path = path_from_c_str!(path);
        e(canonicalize(path).and_then(|path| syscall::unlink(&path))) as c_int
    }

    fn waitpid(mut pid: pid_t, stat_loc: *mut c_int, options: c_int) -> pid_t {
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

                res = Some(e(inner(
                    &mut status,
                    options | sys_wait::WNOHANG | sys_wait::WUNTRACED,
                )));
                if res == Some(0) {
                    // WNOHANG, just pretend ptrace SIGSTOP:ped this
                    status = (syscall::SIGSTOP << 8) | 0x7f;
                    assert!(syscall::wifstopped(status));
                    assert_eq!(syscall::wstopsig(status), syscall::SIGSTOP);
                    res = Some(pid as usize);
                }
            }
        }

        // If ptrace didn't impact this waitpid, proceed *almost* as
        // normal: We still need to add WUNTRACED, but we only return
        // it if (and only if) a ptrace traceme was activated during
        // the wait.
        let res = res.unwrap_or_else(|| loop {
            let res = e(inner(&mut status, options | sys_wait::WUNTRACED));

            // TODO: Also handle special PIDs here
            if !syscall::wifstopped(status) || ptrace::is_traceme(pid) {
                break res;
            }
        });

        // If stat_loc is non-null, set that and the return
        unsafe {
            if !stat_loc.is_null() {
                *stat_loc = status as c_int;
            }
        }
        res as pid_t
    }

    fn write(fd: c_int, buf: &[u8]) -> ssize_t {
        e(syscall::write(fd as usize, buf)) as ssize_t
    }

    fn verify() -> bool {
        // GETPID on Redox is 20, which is WRITEV on Linux
        e(unsafe { syscall::syscall5(syscall::number::SYS_GETPID, !0, !0, !0, !0, !0) }) != !0
    }

    fn exit_thread() -> ! {
        Self::exit(0)
    }
}
