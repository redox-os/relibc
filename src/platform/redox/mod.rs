//! sys/socket implementation, following http://pubs.opengroup.org/onlinepubs/009696699/basedefs/sys/socket.h.html

use alloc::btree_map::BTreeMap;
use cbitset::BitSet;
use core::fmt::Write as WriteFmt;
use core::{mem, ptr, slice};
use spin::{Mutex, MutexGuard, Once};
use syscall::data::Stat as redox_stat;
use syscall::data::TimeSpec as redox_timespec;
use syscall::flag::*;
use syscall::{self, Result};

use c_str::{CStr, CString};
use fs::File;
use io::{self, BufReader, SeekFrom};
use io::prelude::*;
use header::dirent::dirent;
use header::errno::{EIO, EINVAL, ENOSYS};
use header::fcntl;
const MAP_ANON: c_int = 1;
//use header::sys_mman::MAP_ANON;
//use header::sys_resource::rusage;
use header::sys_select::fd_set;
use header::sys_stat::stat;
use header::sys_time::{itimerval, timeval, timezone};
//use header::sys_times::tms;
use header::sys_utsname::utsname;
use header::termios::termios;
use header::time::timespec;
use header::unistd::{F_OK, R_OK, SEEK_SET, W_OK, X_OK};

use super::types::*;
use super::{errno, FileReader, FileWriter, Line, Pal, Read};

mod signal;
mod socket;

static ANONYMOUS_MAPS: Once<Mutex<BTreeMap<usize, usize>>> = Once::new();

fn anonymous_maps() -> MutexGuard<'static, BTreeMap<usize, usize>> {
    ANONYMOUS_MAPS
        .call_once(|| Mutex::new(BTreeMap::new()))
        .lock()
}

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
            Err(_) => unsafe {
                errno = EIO;
                return -1;
            }
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
            // octal has max 7 characters, binary has max two. And we're interested
            // in the 3rd digit
            stat.st_mode >> ((7 / 2) * 2 & 0o7)
        } else if stat.st_gid as usize == gid {
            stat.st_mode >> ((7 / 2) & 0o7)
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
        match syscall::open(path.to_bytes(), O_WRONLY | O_CLOEXEC) {
            Err(err) => e(Err(err)) as c_int,
            Ok(fd) => {
                let res = syscall::fchmod(fd as usize, mode as u16);
                let _ = syscall::close(fd);
                e(res) as c_int
            }
        }
    }

    fn chown(path: &CStr, owner: uid_t, group: gid_t) -> c_int {
        match syscall::open(path.to_bytes(), O_WRONLY | O_CLOEXEC) {
            Err(err) => e(Err(err)) as c_int,
            Ok(fd) => {
                let res = syscall::fchown(fd as usize, owner as u32, group as u32);
                let _ = syscall::close(fd);
                e(res) as c_int
            }
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
        use alloc::Vec;

        let file = match File::open(path, fcntl::O_RDONLY | fcntl::O_CLOEXEC) {
            Ok(file) => file,
            Err(_) => return -1
        };
        let mut file = BufReader::new(file);

        // Count arguments
        let mut len = 0;
        while !(*argv.offset(len)).is_null() {
            len += 1;
        }

        let mut args: Vec<[usize; 2]> = Vec::with_capacity(len as usize);

        // Read shebang (for example #!/bin/sh)
        let mut shebang = [0; 2];
        let mut read = 0;

        while read < 2 {
            match file.read(&mut shebang) {
                Ok(0) => break,
                Ok(i) => read += i,
                Err(_) => return -1
            }
        }

        let mut _interpreter_path = None;
        let mut _interpreter_file = None;
        let mut interpreter_fd = **file.get_ref();

        if &shebang == b"#!" {
            let mut line = Vec::new();
            match file.read_until(b'\n', &mut line) {
                Err(_) => return -1,
                Ok(0) => (),
                Ok(_) => {
                    let mut path = match CString::new(line) {
                        Ok(path) => path,
                        Err(_) => return -1,
                    };
                    match File::open(&path, fcntl::O_RDONLY | fcntl::O_CLOEXEC) {
                        Ok(file) => {
                            interpreter_fd = *file;
                            _interpreter_path = Some(path);
                            _interpreter_file = Some(file);

                            let path_ref = _interpreter_path.as_ref().unwrap();
                            args.push([path_ref.as_ptr() as usize, path_ref.to_bytes().len()]);
                        },
                        Err(_) => return -1,
                    }
                },
            }
        }
        if file.seek(SeekFrom::Start(0)).is_err() {
            return -1;
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

        e(syscall::fexec(interpreter_fd as usize, &args, &envs)) as c_int
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

    fn fsync(fd: c_int) -> c_int {
        e(syscall::fsync(fd as usize)) as c_int
    }

    fn ftruncate(fd: c_int, len: off_t) -> c_int {
        e(syscall::ftruncate(fd as usize, len as usize)) as c_int
    }

    fn futex(addr: *mut c_int, op: c_int, val: c_int) -> c_int {
        match unsafe { syscall::futex(addr as *mut i32, op as usize, val as i32, 0, ptr::null_mut()) } {
            Ok(success) => success as c_int,
            Err(err) => -(err.errno as c_int)
        }
    }

    fn futimens(fd: c_int, times: *const timespec) -> c_int {
        let times = [unsafe { redox_timespec::from(&*times) }, unsafe {
            redox_timespec::from(&*times.offset(1))
        }];
        e(syscall::futimens(fd as usize, &times)) as c_int
    }

    fn utimens(path: &CStr, times: *const timespec) -> c_int {
        match syscall::open(path.to_bytes(), O_STAT | O_CLOEXEC) {
            Err(err) => e(Err(err)) as c_int,
            Ok(fd) => {
                let res = Self::futimens(fd as c_int, times);
                let _ = syscall::close(fd);
                res
            }
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

    fn getdents(fd: c_int, mut dirents: *mut dirent, mut bytes: usize) -> c_int {
        let mut amount = 0;

        let mut buf = [0; 1024];
        let mut bindex = 0;
        let mut blen = 0;

        let mut name = [0; 256];
        let mut nindex = 0;

        loop {
            if bindex >= blen {
                bindex = 0;
                blen = match syscall::read(fd as usize, &mut buf) {
                    Ok(0) => return amount,
                    Ok(n) => n,
                    Err(err) => return -err.errno,
                };
            }

            if buf[bindex] == b'\n' {
                // Put a NUL byte either at the end, or if it's too big, at where it's truncated.
                name[nindex.min(name.len() - 1)] = 0;
                unsafe {
                    *dirents = dirent {
                        d_ino: 0,
                        d_off: 0,
                        d_reclen: mem::size_of::<dirent>() as c_ushort,
                        d_type: 0,
                        d_name: name,
                    };
                    dirents = dirents.offset(1);
                }
                amount += 1;
                if bytes <= mem::size_of::<dirent>() {
                    return amount;
                }
                bytes -= mem::size_of::<dirent>();
            } else {
                if nindex < name.len() {
                    name[nindex] = buf[bindex] as c_char;
                }
                nindex += 1;
                bindex += 1;
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

    fn gethostname(name: *mut c_char, len: size_t) -> c_int {
        fn inner(name: &mut [u8]) -> io::Result<()> {
            let mut file = File::open(
                &CString::new("/etc/hostname").unwrap(),
                fcntl::O_RDONLY | fcntl::O_CLOEXEC
            )?;

            let mut read = 0;
            loop {
                match file.read(&mut name[read..])? {
                    0 => break,
                    n => read += n
                }
            }
            Ok(())
        }

        match inner(unsafe { slice::from_raw_parts_mut(name as *mut u8, len as usize) }) {
            Ok(()) => 0,
            Err(_) => unsafe {
                errno = EIO;
                -1
            }
        }
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
        let flags = O_CREAT | O_EXCL | O_CLOEXEC | O_DIRECTORY | mode as usize & 0o777;
        match syscall::open(path.to_bytes(), flags) {
            Ok(fd) => {
                let _ = syscall::close(fd);
                0
            }
            Err(err) => e(Err(err)) as c_int,
        }
    }

    fn mkfifo(path: &CStr, mode: mode_t) -> c_int {
        let flags = O_CREAT | O_CLOEXEC | MODE_FIFO as usize | mode as usize & 0o777;
        match syscall::open(path.to_bytes(), flags) {
            Ok(fd) => {
                let _ = syscall::close(fd);
                0
            }
            Err(err) => e(Err(err)) as c_int,
        }
    }

    unsafe fn mmap(
        _addr: *mut c_void,
        len: usize,
        _prot: c_int,
        flags: c_int,
        fildes: c_int,
        off: off_t,
    ) -> *mut c_void {
        if flags & MAP_ANON == MAP_ANON {
            let fd = e(syscall::open("memory:", O_STAT | O_CLOEXEC)); // flags don't matter currently
            if fd == !0 {
                return !0 as *mut c_void;
            }

            let addr = e(syscall::fmap(fd, off as usize, len as usize));
            if addr == !0 {
                let _ = syscall::close(fd);
                return !0 as *mut c_void;
            }

            anonymous_maps().insert(addr as usize, fd);
            addr as *mut c_void
        } else {
            e(syscall::fmap(fildes as usize, off as usize, len as usize)) as *mut c_void
        }
    }

    unsafe fn munmap(addr: *mut c_void, _len: usize) -> c_int {
        if e(syscall::funmap(addr as usize)) == !0 {
            return !0;
        }
        if let Some(fd) = anonymous_maps().remove(&(addr as usize)) {
            let _ = syscall::close(fd);
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
            (oflag as usize) | (mode as usize),
        )) as c_int
    }

    fn pipe(fds: &mut [c_int]) -> c_int {
        let mut usize_fds: [usize; 2] = [0; 2];
        let res = e(syscall::pipe2(&mut usize_fds, 0));
        fds[0] = usize_fds[0] as c_int;
        fds[1] = usize_fds[1] as c_int;
        res as c_int
    }

    fn read(fd: c_int, buf: &mut [u8]) -> ssize_t {
        e(syscall::read(fd as usize, buf)) as ssize_t
    }

    fn realpath(pathname: &CStr, out: &mut [u8]) -> c_int {
        let file = match File::open(pathname, fcntl::O_PATH) {
            Ok(fd) => fd,
            Err(_) => return -1
        };

        if out.is_empty() {
            return 0;
        }

        let len = out.len();
        let read = e(syscall::fpath(*file as usize, &mut out[..len-1]));
        if (read as c_int) < 0 {
            return -1;
        }
        out[read as usize] = 0;

        0
    }

    fn rename(oldpath: &CStr, newpath: &CStr) -> c_int {
        match syscall::open(oldpath.to_bytes(), O_WRONLY | O_CLOEXEC) {
            Ok(fd) => {
                let retval = syscall::frename(fd, newpath.to_bytes());
                let _ = syscall::close(fd);
                e(retval) as c_int
            }
            err => e(err) as c_int,
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
        let mut exceptfds = unsafe { exceptfds.as_mut() }.map(|s| BitSet::from_ref(&mut s.fds_bits));

        let event_path = unsafe { CStr::from_bytes_with_nul_unchecked(b"event:\0") };
        let mut event_file = match File::open(event_path, fcntl::O_RDWR | fcntl::O_CLOEXEC) {
            Ok(file) => file,
            Err(_) => return -1,
        };

        for fd in 0..nfds as usize {
            macro_rules! register {
                ($fd:expr, $flags:expr) => {
                    if event_file.write(&syscall::Event { id: $fd, flags: $flags, data: 0, }).is_err() {
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
            let mut timeout_file = match File::open(&timeout_path, fcntl::O_RDWR | fcntl::O_CLOEXEC) {
                Ok(file) => file,
                Err(_) => return -1,
            };

            if event_file.write(&syscall::Event {
                id: *timeout_file as usize,
                flags: syscall::EVENT_READ,
                data: TIMEOUT_TOKEN,
            }).is_err() {
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
            let mut events = unsafe { slice::from_raw_parts_mut(
                &mut events as *mut _ as *mut u8,
                mem::size_of::<syscall::Event>() * events.len()
            ) };
            match event_file.read(&mut events) {
                Ok(i) => i / mem::size_of::<syscall::Event>(),
                Err(_) => return -1
            }
        };

        let mut total = 0;

        if let Some(ref mut set) = readfds { set.clear(); }
        if let Some(ref mut set) = writefds { set.clear(); }
        if let Some(ref mut set) = exceptfds { set.clear(); }

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
