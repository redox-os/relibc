use core::ptr;
use core::slice;
use core::mem;
use syscall;
use syscall::flag::*;
use syscall::data::TimeSpec as redox_timespec;
use syscall::data::Stat as redox_stat;

use c_str;
use errno;
use types::*;

pub fn e(sys: Result<usize, syscall::Error>) -> usize {
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

pub fn brk(addr: *const c_void) -> c_int {
    e(unsafe { syscall::brk(addr as usize) }) as c_int
}

pub fn chdir(path: *const c_char) -> c_int {
    let path = unsafe { c_str(path) };
    e(syscall::chdir(path)) as c_int
}

pub fn chmod(path: *const c_char, mode: mode_t) -> c_int {
    let path = unsafe { c_str(path) };
    match syscall::open(path, O_WRONLY) {
        Err(err) => e(Err(err)) as c_int,
        Ok(fd) => {
            let res = syscall::fchmod(fd as usize, mode);
            let _ = syscall::close(fd);
            e(res) as c_int
        }
    }
}

pub fn chown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    let path = unsafe { c_str(path) };
    match syscall::open(path, O_WRONLY) {
        Err(err) => e(Err(err)) as c_int,
        Ok(fd) => {
           let res = syscall::fchown(fd as usize, owner as u32, group as u32);
           let _ = syscall::close(fd);
           e(res) as c_int
        }
    }
}

pub fn close(fd: c_int) -> c_int {
    e(syscall::close(fd as usize)) as c_int
}

pub fn dup(fd: c_int) -> c_int {
    e(syscall::dup(fd as usize, &[])) as c_int
}

pub fn dup2(fd1: c_int, fd2: c_int) -> c_int {
    e(syscall::dup2(fd1 as usize, fd2 as usize, &[])) as c_int
}

pub fn exit(status: c_int) -> ! {
    let _ = syscall::exit(status as usize);
    loop {}
}

pub fn fchdir(fd: c_int) -> c_int {
    let path: &mut [u8] = &mut [0; 4096];
    if e(syscall::fpath(fd as usize, path)) == !0 {
        !0
    } else {
        e(syscall::chdir(path)) as c_int
    }
}

pub fn fchmod(fd: c_int, mode: mode_t) -> c_int {
    e(syscall::fchmod(fd as usize, mode)) as c_int
}

pub fn fchown(fd: c_int, owner: uid_t, group: gid_t) -> c_int {
    e(syscall::fchown(fd as usize, owner as u32, group as u32)) as c_int
}

pub fn fcntl(fd: c_int, cmd: c_int, args: c_int) -> c_int {
    e(syscall::fcntl(fd as usize, cmd as usize, args as usize)) as c_int
}

pub fn fork() -> pid_t {
    e(unsafe { syscall::clone(0) }) as pid_t
}

pub fn fstat(fildes: c_int, buf: *mut stat) -> c_int {
    let mut redox_buf: redox_stat = redox_stat::default(); 
    match e(syscall::fstat(fildes as usize, &mut redox_buf)) {
       0 => {
                unsafe {
                    if !buf.is_null() {
                        (*buf).st_dev = redox_buf.st_dev as dev_t;
                        (*buf).st_ino = redox_buf.st_ino as ino_t;
                        (*buf).st_nlink = redox_buf.st_nlink as nlink_t;
                        (*buf).st_mode = redox_buf.st_mode;
                        (*buf).st_uid = redox_buf.st_uid as uid_t;
                        (*buf).st_gid = redox_buf.st_gid as gid_t;
                        // TODO st_rdev
                        (*buf).st_rdev = 0;
                        (*buf).st_size = redox_buf.st_size as off_t;
                        (*buf).st_blksize = redox_buf.st_blksize as blksize_t;
                        (*buf).st_atim = redox_buf.st_atime as time_t;
                        (*buf).st_mtim = redox_buf.st_mtime as time_t;
                        (*buf).st_ctim = redox_buf.st_ctime as time_t;
                    }
                }
                0
            },
        _ => -1,
    }
}

pub fn fsync(fd: c_int) -> c_int {
    e(syscall::fsync(fd as usize)) as c_int
}

pub fn ftruncate(fd: c_int, len: off_t) -> c_int {
    e(syscall::ftruncate(fd as usize, len as usize)) as c_int
}

pub fn getcwd(buf: *mut c_char, size: size_t) -> *mut c_char {
    let buf_slice = unsafe { slice::from_raw_parts_mut(buf as *mut u8, size as usize) };
    if e(syscall::getcwd(buf_slice)) == !0 {
        ptr::null_mut()
    } else {
        buf
    }
}

pub fn getegid() -> gid_t {
    e(syscall::getegid()) as gid_t
}

pub fn geteuid() -> uid_t {
    e(syscall::geteuid()) as uid_t
}

pub fn getgid() -> gid_t {
    e(syscall::getgid()) as gid_t
}

pub fn getpgid(pid: pid_t) -> pid_t {
    e(syscall::getpgid(pid as usize)) as pid_t
}

pub fn getpid() -> pid_t {
    e(syscall::getpid()) as pid_t
}

pub fn getppid() -> pid_t {
    e(syscall::getppid()) as pid_t
}

pub fn getuid() -> uid_t {
    e(syscall::getuid()) as pid_t
}

pub fn kill(pid: pid_t, sig: c_int) -> c_int {
    e(syscall::kill(pid, sig as usize)) as c_int
}

pub fn killpg(pgrp: pid_t, sig: c_int) -> c_int {
    e(syscall::kill(-(pgrp as isize) as pid_t, sig as usize)) as c_int
}

pub fn link(path1: *const c_char, path2: *const c_char) -> c_int {
    let path1 = unsafe { c_str(path1) };
    let path2 = unsafe { c_str(path2) };
    e(unsafe { syscall::link(path1.as_ptr(), path2.as_ptr()) }) as c_int
}

pub fn lseek(fd: c_int, offset: off_t, whence: c_int) -> off_t {
    e(syscall::lseek(
        fd as usize,
        offset as isize,
        whence as usize,
    )) as off_t
}

pub fn lstat(path: *const c_char, buf: *mut stat) -> c_int {
    let path = unsafe { c_str(path) };
    match syscall::open(path, O_RDONLY | O_NOFOLLOW) {
        Err(err) => e(Err(err)) as c_int,
        Ok(fd) => {
            let res = fstat(fd as i32, buf);
            let _ = syscall::close(fd);
            res
        }
    }

pub fn mkdir(path: *const c_char, mode: mode_t) -> c_int {
    let flags = O_CREAT | O_EXCL | O_CLOEXEC | O_DIRECTORY | mode as usize & 0o777;
    let path = unsafe { c_str(path) };
    match syscall::open(path, flags) {
        Ok(fd) => {
            let _ = syscall::close(fd);
            0
        }
        Err(err) => e(Err(err)) as c_int,
    }
}

pub fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> c_int {
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

pub fn open(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    let path = unsafe { c_str(path) };
    e(syscall::open(path, (oflag as usize) | (mode as usize))) as c_int
}

pub fn pipe(mut fds: [c_int; 2]) -> c_int {
    let mut usize_fds: [usize; 2] = [0; 2];
    let res = e(syscall::pipe2(&mut usize_fds, 0));
    fds[0] = usize_fds[0] as c_int;
    fds[1] = usize_fds[1] as c_int;
    res as c_int
}

pub fn read(fd: c_int, buf: &mut [u8]) -> ssize_t {
    e(syscall::read(fd as usize, buf)) as ssize_t
}

pub fn rmdir(path: *const c_char) -> c_int {
    let path = unsafe { c_str(path) };
    e(syscall::rmdir(path)) as c_int
}

pub fn setpgid(pid: pid_t, pgid: pid_t) -> c_int {
    e(syscall::setpgid(pid as usize, pgid as usize)) as c_int
}

pub fn setregid(rgid: gid_t, egid: gid_t) -> c_int {
    e(syscall::setregid(rgid as usize, egid as usize)) as c_int
}

pub fn setreuid(ruid: uid_t, euid: uid_t) -> c_int {
    e(syscall::setreuid(ruid as usize, euid as usize)) as c_int
}

pub fn stat(path: *const c_char, buf: *mut stat) -> c_int {
    let path = unsafe { c_str(path) };
    match syscall::open(path, O_RDONLY) {
        Err(err) => e(Err(err)) as c_int,
        Ok(fd) => {
            let res = fstat(fd as i32, buf);
            let _ = syscall::close(fd);
            res
        }
    }
}

pub fn unlink(path: *const c_char) -> c_int {
    let path = unsafe { c_str(path) };
    e(syscall::unlink(path)) as c_int
}

pub fn waitpid(pid: pid_t, stat_loc: *mut c_int, options: c_int) -> pid_t {
    unsafe {
        let mut temp: usize = 0;
        let res = e(syscall::waitpid(pid as usize, &mut temp, options as usize));
        if !stat_loc.is_null() {
            *stat_loc = temp as c_int;
        }
        res
    }
}

pub fn write(fd: c_int, buf: &[u8]) -> ssize_t {
    e(syscall::write(fd as usize, buf)) as ssize_t
}

pub fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> c_int {
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
