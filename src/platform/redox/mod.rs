//! sys/socket implementation, following http://pubs.opengroup.org/onlinepubs/009696699/basedefs/sys/socket.h.html

use cbitset::BitSet;
use core::result::Result as CoreResult;
use core::{mem, ptr, slice};
use syscall::data::Map;
use syscall::data::Stat as redox_stat;
use syscall::data::StatVfs as redox_statvfs;
use syscall::data::TimeSpec as redox_timespec;
use syscall::{self, Result};

use c_str::{CStr, CString};
use fs::File;
use header::dirent::dirent;
use header::errno::{EINVAL, EIO, EPERM};
use header::fcntl;
use header::poll::{self, nfds_t, pollfd};
use header::sys_mman::MAP_ANON;
use header::sys_select::fd_set;
use header::sys_stat::stat;
use header::sys_statvfs::statvfs;
use header::sys_time::{timeval, timezone};
use header::sys_utsname::{utsname, UTSLENGTH};
use header::termios::termios;
use header::time::timespec;
use header::unistd::{F_OK, R_OK, W_OK, X_OK};
use io::prelude::*;
use io::{self, BufReader, SeekFrom};

use super::types::*;
use super::{errno, Pal, Read};

mod extra;
mod pte;
mod signal;
mod socket;

fn e(sys: Result<usize>) -> usize {
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
    fn access(path: &CStr, mode: c_int) -> c_int {
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
        unsafe { syscall::brk(addr as usize).unwrap_or(0) as *mut c_void }
    }

    fn chdir(path: &CStr) -> c_int {
        e(syscall::chdir(path.to_bytes())) as c_int
    }

    fn chmod(path: &CStr, mode: mode_t) -> c_int {
        match File::open(path, fcntl::O_PATH | fcntl::O_CLOEXEC) {
            Ok(file) => Self::fchmod(*file, mode),
            Err(_) => -1,
        }
    }

    fn chown(path: &CStr, owner: uid_t, group: gid_t) -> c_int {
        match File::open(path, fcntl::O_PATH | fcntl::O_CLOEXEC) {
            Ok(file) => Self::fchown(*file, owner, group),
            Err(_) => -1,
        }
    }

    fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> c_int {
        let mut redox_tp = unsafe { redox_timespec::from(&*tp) };
        match e(syscall::clock_gettime(clk_id as usize, &mut redox_tp)) as c_int {
            -1 => -1,
            _ => {
                unsafe {
                    (*tp).tv_sec = redox_tp.tv_sec;
                    (*tp).tv_nsec = redox_tp.tv_nsec as i64;
                };
                0
            }
        }
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

    unsafe fn execve(
        path: &CStr,
        mut argv: *const *mut c_char,
        mut envp: *const *mut c_char,
    ) -> c_int {
        use alloc::vec::Vec;

        let mut file = match File::open(path, fcntl::O_RDONLY | fcntl::O_CLOEXEC) {
            Ok(file) => file,
            Err(_) => return -1,
        };
        let fd = *file as usize;

        // Count arguments
        let mut len = 0;
        while !(*argv.offset(len)).is_null() {
            len += 1;
        }

        let mut args: Vec<[usize; 2]> = Vec::with_capacity(len as usize);

        // Read shebang (for example #!/bin/sh)
        let interpreter = {
            let mut reader = BufReader::new(&mut file);

            let mut shebang = [0; 2];
            let mut read = 0;

            while read < 2 {
                match reader.read(&mut shebang) {
                    Ok(0) => break,
                    Ok(i) => read += i,
                    Err(_) => return -1,
                }
            }

            if &shebang == b"#!" {
                // So, this file is interpreted.
                // That means the actual file descriptor passed to `fexec` won't be this file.
                // So we need to check ourselves that this file is actually be executable.

                let mut stat = redox_stat::default();
                if e(syscall::fstat(fd, &mut stat)) == !0 {
                    return -1;
                }
                let uid = e(syscall::getuid());
                if uid == !0 {
                    return -1;
                }
                let gid = e(syscall::getuid());
                if gid == !0 {
                    return -1;
                }

                let mode = if uid == stat.st_uid as usize {
                    (stat.st_mode >> 3 * 2) & 0o7
                } else if gid == stat.st_gid as usize {
                    (stat.st_mode >> 3 * 1) & 0o7
                } else {
                    stat.st_mode & 0o7
                };

                if mode & 0o1 == 0o0 {
                    errno = EPERM;
                    return -1;
                }

                // Then, read the actual interpreter:
                let mut interpreter = Vec::new();
                match reader.read_until(b'\n', &mut interpreter) {
                    Err(_) => return -1,
                    Ok(_) => {
                        if interpreter.ends_with(&[b'\n']) {
                            interpreter.pop().unwrap();
                        }
                        // TODO: Returning the interpreter here is actually a
                        // hack. Preferrably we should reassign `file =`
                        // directly from here. Just wait until NLL comes
                        // around...
                        Some(interpreter)
                    }
                }
            } else {
                None
            }
        };
        let mut _interpreter_path = None;
        if let Some(interpreter) = interpreter {
            let mut cstring = match CString::new(interpreter) {
                Ok(cstring) => cstring,
                Err(_) => return -1,
            };
            file = match File::open(&cstring, fcntl::O_RDONLY | fcntl::O_CLOEXEC) {
                Ok(file) => file,
                Err(_) => return -1,
            };

            // Make sure path is kept alive long enough, and push it to the arguments
            _interpreter_path = Some(cstring);
            let path_ref = _interpreter_path.as_ref().unwrap();
            args.push([path_ref.as_ptr() as usize, path_ref.to_bytes().len()]);
        } else {
            if file.seek(SeekFrom::Start(0)).is_err() {
                return -1;
            }
        }

        // Arguments
        while !(*argv).is_null() {
            let arg = *argv;

            let mut len = 0;
            while *arg.offset(len) != 0 {
                len += 1;
            }
            args.push([arg as usize, len as usize]);
            argv = argv.offset(1);
        }

        // Environment variables
        let mut len = 0;
        while !(*envp.offset(len)).is_null() {
            len += 1;
        }

        let mut envs: Vec<[usize; 2]> = Vec::with_capacity(len as usize);
        while !(*envp).is_null() {
            let env = *envp;

            let mut len = 0;
            while *env.offset(len) != 0 {
                len += 1;
            }
            envs.push([env as usize, len as usize]);
            envp = envp.offset(1);
        }

        e(syscall::fexec(*file as usize, &args, &envs)) as c_int
    }

    fn fchdir(fd: c_int) -> c_int {
        let path: &mut [u8] = &mut [0; 4096];
        if e(syscall::fpath(fd as usize, path)) == !0 {
            !0
        } else {
            e(syscall::chdir(path)) as c_int
        }
    }

    fn fchmod(fd: c_int, mode: mode_t) -> c_int {
        e(syscall::fchmod(fd as usize, mode as u16)) as c_int
    }

    fn fchown(fd: c_int, owner: uid_t, group: gid_t) -> c_int {
        e(syscall::fchown(fd as usize, owner as u32, group as u32)) as c_int
    }

    fn fcntl(fd: c_int, cmd: c_int, args: c_int) -> c_int {
        e(syscall::fcntl(fd as usize, cmd as usize, args as usize)) as c_int
    }

    fn flock(_fd: c_int, _operation: c_int) -> c_int {
        // TODO: Redox does not have file locking yet
        0
    }

    fn fork() -> pid_t {
        e(unsafe { syscall::clone(0) }) as pid_t
    }

    fn fstat(fildes: c_int, buf: *mut stat) -> c_int {
        let mut redox_buf: redox_stat = redox_stat::default();
        match e(syscall::fstat(fildes as usize, &mut redox_buf)) {
            0 => {
                unsafe {
                    if !buf.is_null() {
                        (*buf).st_dev = redox_buf.st_dev as dev_t;
                        (*buf).st_ino = redox_buf.st_ino as ino_t;
                        (*buf).st_nlink = redox_buf.st_nlink as nlink_t;
                        (*buf).st_mode = redox_buf.st_mode as mode_t;
                        (*buf).st_uid = redox_buf.st_uid as uid_t;
                        (*buf).st_gid = redox_buf.st_gid as gid_t;
                        // TODO st_rdev
                        (*buf).st_rdev = 0;
                        (*buf).st_size = redox_buf.st_size as off_t;
                        (*buf).st_blksize = redox_buf.st_blksize as blksize_t;
                        (*buf).st_atim = timespec {
                            tv_sec: redox_buf.st_atime as time_t,
                            tv_nsec: redox_buf.st_atime_nsec as c_long,
                        };
                        (*buf).st_mtim = timespec {
                            tv_sec: redox_buf.st_mtime as time_t,
                            tv_nsec: redox_buf.st_mtime_nsec as c_long,
                        };
                        (*buf).st_ctim = timespec {
                            tv_sec: redox_buf.st_ctime as time_t,
                            tv_nsec: redox_buf.st_ctime_nsec as c_long,
                        };
                    }
                }
                0
            }
            _ => -1,
        }
    }

    fn fstatvfs(fildes: c_int, buf: *mut statvfs) -> c_int {
        let mut kbuf: redox_statvfs = redox_statvfs::default();
        match e(syscall::fstatvfs(fildes as usize, &mut kbuf)) {
            0 => {
                unsafe {
                    if !buf.is_null() {
                        (*buf).f_bsize = kbuf.f_bsize as c_ulong;
                        (*buf).f_frsize = kbuf.f_bsize as c_ulong;
                        (*buf).f_blocks = kbuf.f_blocks;
                        (*buf).f_bfree = kbuf.f_bfree;
                        (*buf).f_bavail = kbuf.f_bavail;
                        //TODO
                        (*buf).f_files = 0;
                        (*buf).f_ffree = 0;
                        (*buf).f_favail = 0;
                        (*buf).f_fsid = 0;
                        (*buf).f_flag = 0;
                        (*buf).f_namemax = 0;
                    }
                }
                0
            }
            _ => -1,
        }
    }

    fn fsync(fd: c_int) -> c_int {
        e(syscall::fsync(fd as usize)) as c_int
    }

    fn ftruncate(fd: c_int, len: off_t) -> c_int {
        e(syscall::ftruncate(fd as usize, len as usize)) as c_int
    }

    fn futex(addr: *mut c_int, op: c_int, val: c_int) -> c_int {
        match unsafe {
            syscall::futex(
                addr as *mut i32,
                op as usize,
                val as i32,
                0,
                ptr::null_mut(),
            )
        } {
            Ok(success) => success as c_int,
            Err(err) => -(err.errno as c_int),
        }
    }

    fn futimens(fd: c_int, times: *const timespec) -> c_int {
        let times = [unsafe { redox_timespec::from(&*times) }, unsafe {
            redox_timespec::from(&*times.offset(1))
        }];
        e(syscall::futimens(fd as usize, &times)) as c_int
    }

    fn utimens(path: &CStr, times: *const timespec) -> c_int {
        match File::open(path, fcntl::O_PATH | fcntl::O_CLOEXEC) {
            Ok(file) => Self::futimens(*file, times),
            Err(_) => -1,
        }
    }

    fn getcwd(buf: *mut c_char, size: size_t) -> *mut c_char {
        let buf_slice = unsafe { slice::from_raw_parts_mut(buf as *mut u8, size as usize - 1) };
        let read = e(syscall::getcwd(buf_slice));
        if read == !0 {
            ptr::null_mut()
        } else {
            unsafe {
                *buf.offset(read as isize + 1) = 0;
            }
            buf
        }
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

    fn getpagesize() -> c_int {
        4096
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

    fn isatty(fd: c_int) -> c_int {
        syscall::dup(fd as usize, b"termios")
            .map(|fd| {
                let _ = syscall::close(fd);
                1
            })
            .unwrap_or(0)
    }

    fn link(path1: &CStr, path2: &CStr) -> c_int {
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

    fn mkdir(path: &CStr, mode: mode_t) -> c_int {
        match File::create(path, fcntl::O_DIRECTORY | fcntl::O_EXCL | fcntl::O_CLOEXEC, 0o777) {
            Ok(_fd) => 0,
            Err(_) => -1,
        }
    }

    fn mkfifo(path: &CStr, mode: mode_t) -> c_int {
        match File::create(path, fcntl::O_CREAT | fcntl::O_CLOEXEC, syscall::MODE_FIFO as mode_t | (mode & 0o777)) {
            Ok(fd) => 0,
            Err(_) => -1,
        }
    }

    unsafe fn mmap(
        _addr: *mut c_void,
        len: usize,
        prot: c_int,
        flags: c_int,
        fildes: c_int,
        off: off_t,
    ) -> *mut c_void {
        let map = Map {
            offset: off as usize,
            size: len,
            flags: ((prot as usize) << 16) | ((flags as usize) & 0xFFFF)
        };

        if flags & MAP_ANON == MAP_ANON {
            let fd = e(syscall::open("memory:", syscall::O_STAT | syscall::O_CLOEXEC)); // flags don't matter currently
            if fd == !0 {
                return !0 as *mut c_void;
            }

            let addr = e(syscall::fmap(fd, &map)) as *mut c_void;

            let _ = syscall::close(fd);

            addr
        } else {
            e(syscall::fmap(fildes as usize, &map)) as *mut c_void
        }
    }

    unsafe fn munmap(addr: *mut c_void, _len: usize) -> c_int {
        if e(syscall::funmap(addr as usize)) == !0 {
            return !0;
        }
        0
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
                        (*rmtp).tv_sec = redox_rmtp.tv_sec;
                        (*rmtp).tv_nsec = redox_rmtp.tv_nsec as i64;
                    }
                }
                0
            }
        }
    }

    fn open(path: &CStr, oflag: c_int, mode: mode_t) -> c_int {
        e(syscall::open(
            path.to_bytes(),
            ((oflag as usize) << 16) | ((mode as usize) & 0xFFFF),
        )) as c_int
    }

    fn pipe(fds: &mut [c_int]) -> c_int {
        let mut usize_fds: [usize; 2] = [0; 2];
        let res = e(syscall::pipe2(&mut usize_fds, 0));
        fds[0] = usize_fds[0] as c_int;
        fds[1] = usize_fds[1] as c_int;
        res as c_int
    }

    fn poll(fds: *mut pollfd, nfds: nfds_t, timeout: c_int) -> c_int {
        let fds = unsafe { slice::from_raw_parts_mut(fds, nfds as usize) };

        let event_path = c_str!("event:");
        let mut event_file = match File::open(event_path, fcntl::O_RDWR | fcntl::O_CLOEXEC) {
            Ok(file) => file,
            Err(_) => return -1,
        };

        for fd in fds.iter_mut() {
            let mut flags = 0;

            if fd.events & poll::POLLIN > 0 {
                flags |= syscall::EVENT_READ;
            }

            if fd.events & poll::POLLOUT > 0 {
                flags |= syscall::EVENT_WRITE;
            }

            fd.revents = 0;

            if fd.fd >= 0 && flags > 0 {
                if event_file
                    .write(&syscall::Event {
                        id: fd.fd as usize,
                        flags: flags,
                        data: 0,
                    })
                    .is_err()
                {
                    return -1;
                }
            }
        }

        const TIMEOUT_TOKEN: usize = 1;

        let timeout_file = if timeout < 0 {
            None
        } else {
            let timeout_path = unsafe {
                CString::from_vec_unchecked(
                    format!("time:{}", syscall::CLOCK_MONOTONIC).into_bytes(),
                )
            };
            let mut timeout_file = match File::open(&timeout_path, fcntl::O_RDWR | fcntl::O_CLOEXEC)
            {
                Ok(file) => file,
                Err(_) => return -1,
            };

            if event_file
                .write(&syscall::Event {
                    id: *timeout_file as usize,
                    flags: syscall::EVENT_READ,
                    data: TIMEOUT_TOKEN,
                })
                .is_err()
            {
                return -1;
            }

            let mut time = syscall::TimeSpec::default();
            if timeout_file.read(&mut time).is_err() {
                return -1;
            }

            time.tv_nsec += timeout * 1000000;
            while time.tv_nsec >= 1000000000 {
                time.tv_sec += 1;
                time.tv_nsec -= 1000000000;
            }

            if timeout_file.write(&time).is_err() {
                return -1;
            }

            Some(timeout_file)
        };

        let mut events = [syscall::Event::default(); 32];
        let read = {
            let mut events = unsafe {
                slice::from_raw_parts_mut(
                    &mut events as *mut _ as *mut u8,
                    mem::size_of::<syscall::Event>() * events.len(),
                )
            };
            match event_file.read(&mut events) {
                Ok(i) => i / mem::size_of::<syscall::Event>(),
                Err(_) => return -1,
            }
        };

        for event in &events[..read] {
            if event.data == TIMEOUT_TOKEN {
                continue;
            }

            for fd in fds.iter_mut() {
                if event.id == fd.fd as usize {
                    if event.flags & syscall::EVENT_READ > 0 {
                        fd.revents |= poll::POLLIN;
                    }

                    if event.flags & syscall::EVENT_WRITE > 0 {
                        fd.revents |= poll::POLLOUT;
                    }
                }
            }
        }

        let mut total = 0;

        for fd in fds.iter_mut() {
            if fd.revents > 0 {
                total += 1;
            }
        }

        total
    }

    fn read(fd: c_int, buf: &mut [u8]) -> ssize_t {
        e(syscall::read(fd as usize, buf)) as ssize_t
    }

    fn readlink(pathname: &CStr, out: &mut [u8]) -> ssize_t {
        let file = match File::open(pathname, fcntl::O_PATH | fcntl::O_SYMLINK | fcntl::O_CLOEXEC) {
            Ok(ok) => ok,
            Err(_) => return -1,
        };

        if out.is_empty() {
            return 0;
        }

        let len = out.len();
        let read = e(syscall::fpath(*file as usize, &mut out[..len - 1]));
        if (read as c_int) < 0 {
            return -1;
        }
        out[read as usize] = 0;

        0
    }

    fn realpath(pathname: &CStr, out: &mut [u8]) -> c_int {
        let file = match File::open(pathname, fcntl::O_PATH | fcntl::O_CLOEXEC) {
            Ok(ok) => ok,
            Err(_) => return -1,
        };

        if out.is_empty() {
            return 0;
        }

        let len = out.len();
        let read = e(syscall::fpath(*file as usize, &mut out[..len - 1]));
        if (read as c_int) < 0 {
            return -1;
        }
        out[read as usize] = 0;

        0
    }

    fn rename(oldpath: &CStr, newpath: &CStr) -> c_int {
        match File::open(oldpath, fcntl::O_WRONLY | fcntl::O_CLOEXEC) {
            Ok(file) => e(syscall::frename(*file as usize, newpath.to_bytes())) as c_int,
            Err(_) => -1,
        }
    }

    fn rmdir(path: &CStr) -> c_int {
        e(syscall::rmdir(path.to_bytes())) as c_int
    }

    fn select(
        nfds: c_int,
        readfds: *mut fd_set,
        writefds: *mut fd_set,
        exceptfds: *mut fd_set,
        timeout: *mut timeval,
    ) -> c_int {
        let mut readfds = unsafe { readfds.as_mut() }.map(|s| BitSet::from_ref(&mut s.fds_bits));
        let mut writefds = unsafe { writefds.as_mut() }.map(|s| BitSet::from_ref(&mut s.fds_bits));
        let mut exceptfds =
            unsafe { exceptfds.as_mut() }.map(|s| BitSet::from_ref(&mut s.fds_bits));

        let event_path = c_str!("event:");
        let mut event_file = match File::open(event_path, fcntl::O_RDWR | fcntl::O_CLOEXEC) {
            Ok(file) => file,
            Err(_) => return -1,
        };

        for fd in 0..nfds as usize {
            macro_rules! register {
                ($fd:expr, $flags:expr) => {
                    if event_file
                        .write(&syscall::Event {
                            id: $fd,
                            flags: $flags,
                            data: 0,
                        })
                        .is_err()
                    {
                        return -1;
                    }
                };
            }
            if readfds.as_mut().map(|s| s.contains(fd)).unwrap_or(false) {
                register!(fd, syscall::EVENT_READ);
            }
            if writefds.as_mut().map(|s| s.contains(fd)).unwrap_or(false) {
                register!(fd, syscall::EVENT_WRITE);
            }
        }

        const TIMEOUT_TOKEN: usize = 1;

        let timeout_file = if timeout.is_null() {
            None
        } else {
            let timeout = unsafe { &*timeout };

            let timeout_path = unsafe {
                CString::from_vec_unchecked(
                    format!("time:{}", syscall::CLOCK_MONOTONIC).into_bytes(),
                )
            };
            let mut timeout_file = match File::open(&timeout_path, fcntl::O_RDWR | fcntl::O_CLOEXEC)
            {
                Ok(file) => file,
                Err(_) => return -1,
            };

            if event_file
                .write(&syscall::Event {
                    id: *timeout_file as usize,
                    flags: syscall::EVENT_READ,
                    data: TIMEOUT_TOKEN,
                })
                .is_err()
            {
                return -1;
            }

            let mut time = syscall::TimeSpec::default();
            if timeout_file.read(&mut time).is_err() {
                return -1;
            }

            time.tv_sec += timeout.tv_sec;
            time.tv_nsec += timeout.tv_usec * 1000;
            while time.tv_nsec >= 1000000000 {
                time.tv_sec += 1;
                time.tv_nsec -= 1000000000;
            }

            if timeout_file.write(&time).is_err() {
                return -1;
            }

            Some(timeout_file)
        };

        let mut events = [syscall::Event::default(); 32];
        let read = {
            let mut events = unsafe {
                slice::from_raw_parts_mut(
                    &mut events as *mut _ as *mut u8,
                    mem::size_of::<syscall::Event>() * events.len(),
                )
            };
            match event_file.read(&mut events) {
                Ok(i) => i / mem::size_of::<syscall::Event>(),
                Err(_) => return -1,
            }
        };

        let mut total = 0;

        if let Some(ref mut set) = readfds {
            set.clear();
        }
        if let Some(ref mut set) = writefds {
            set.clear();
        }
        if let Some(ref mut set) = exceptfds {
            set.clear();
        }

        for event in &events[..read] {
            if event.data == TIMEOUT_TOKEN {
                continue;
            }

            if event.flags & syscall::EVENT_READ == syscall::EVENT_READ {
                if let Some(ref mut set) = readfds {
                    set.insert(event.id);
                }
                total += 1;
            }
            if event.flags & syscall::EVENT_WRITE == syscall::EVENT_WRITE {
                if let Some(ref mut set) = writefds {
                    set.insert(event.id);
                }
                total += 1;
            }
        }

        total
    }

    fn setpgid(pid: pid_t, pgid: pid_t) -> c_int {
        e(syscall::setpgid(pid as usize, pgid as usize)) as c_int
    }

    fn setregid(rgid: gid_t, egid: gid_t) -> c_int {
        e(syscall::setregid(rgid as usize, egid as usize)) as c_int
    }

    fn setreuid(ruid: uid_t, euid: uid_t) -> c_int {
        e(syscall::setreuid(ruid as usize, euid as usize)) as c_int
    }

    fn symlink(path1: &CStr, path2: &CStr) -> c_int {
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

    fn tcgetattr(fd: c_int, out: *mut termios) -> c_int {
        let dup = e(syscall::dup(fd as usize, b"termios"));
        if dup == !0 {
            return -1;
        }

        let read = e(syscall::read(dup, unsafe {
            slice::from_raw_parts_mut(out as *mut u8, mem::size_of::<termios>())
        }));
        let _ = syscall::close(dup);

        if read == !0 {
            return -1;
        }
        0
    }

    fn tcsetattr(fd: c_int, _act: c_int, value: *const termios) -> c_int {
        let dup = e(syscall::dup(fd as usize, b"termios"));
        if dup == !0 {
            return -1;
        }

        let write = e(syscall::write(dup, unsafe {
            slice::from_raw_parts(value as *const u8, mem::size_of::<termios>())
        }));
        let _ = syscall::close(dup);

        if write == !0 {
            return -1;
        }
        0
    }

    fn umask(mask: mode_t) -> mode_t {
        e(syscall::umask(mask as usize)) as mode_t
    }

    fn uname(utsname: *mut utsname) -> c_int {
        fn gethostname(name: &mut [u8]) -> io::Result<()> {
            if name.is_empty() {
                return Ok(())
            }

            let mut file = File::open(
                &CString::new("/etc/hostname").unwrap(),
                fcntl::O_RDONLY | fcntl::O_CLOEXEC,
            )?;

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
                    (*utsname).nodename.len()
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

    fn unlink(path: &CStr) -> c_int {
        e(syscall::unlink(path.to_bytes())) as c_int
    }

    fn waitpid(mut pid: pid_t, stat_loc: *mut c_int, options: c_int) -> pid_t {
        if pid == !0 {
            pid = 0;
        }
        unsafe {
            let mut temp: usize = 0;
            let res = e(syscall::waitpid(pid as usize, &mut temp, options as usize));
            if !stat_loc.is_null() {
                *stat_loc = temp as c_int;
            }
            res as pid_t
        }
    }

    fn write(fd: c_int, buf: &[u8]) -> ssize_t {
        e(syscall::write(fd as usize, buf)) as ssize_t
    }
}
