use core::ptr;
use core::slice;
use core::mem;
use alloc::Vec;
use syscall;
use syscall::flag::*;
use syscall::data::TimeSpec as redox_timespec;

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

pub fn chown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    let path = unsafe { c_str(path) };
    let fd = syscall::open(path, 0x0001).unwrap();
    e(syscall::fchown(fd as usize, owner as u32, group as u32)) as c_int
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

pub fn execve(path: *const c_char, argv: *const *mut c_char, envp: *const *mut c_char) -> c_int {
    unsafe {
        let mut env = envp;
        while !(*env).is_null() {
            let slice = c_str(*env);
            // Should always contain a =, but worth checking
            if let Some(sep) = slice.iter().position(|&c| c == b'=') {
                // If the environment variable has no name, do not attempt
                // to add it to the env.
                if sep > 0 {
                    let mut path = b"env:".to_vec();
                    path.extend_from_slice(&slice[..sep]);
                    match syscall::open(&path, O_WRONLY | O_CREAT) {
                        Ok(fd) => {
                            // If the environment variable has no value, there
                            // is no need to write anything to the env scheme.
                            if sep + 1 < slice.len() {
                                let n = match syscall::write(fd, &slice[sep + 1..]) {
                                    Ok(n) => n,
                                    err => {
                                        return e(err) as c_int;
                                    }
                                };
                            }
                            // Cleanup after adding the variable.
                            match syscall::close(fd) {
                                Ok(_) => (),
                                err => {
                                    return e(err) as c_int;
                                }
                            }
                        }
                        err => {
                            return e(err) as c_int;
                        }
                    }
                }
            }
            env = env.offset(1);
        }

        let mut args: Vec<[usize; 2]> = Vec::new();
        let mut arg = argv;
        while !(*arg).is_null() {
            args.push([*arg as usize, c_str(*arg).len()]);
            arg = arg.offset(1);
        }

        e(syscall::execve(c_str(path), &args)) as c_int
    }
}

pub fn exit(status: c_int) -> ! {
    syscall::exit(status as usize);
    loop {}
}

pub fn fchown(fd: c_int, owner: uid_t, group: gid_t) -> c_int {
    e(syscall::fchown(fd as usize, owner as u32, group as u32)) as c_int
}

pub fn fchdir(fd: c_int) -> c_int {
    let path: &mut [u8] = &mut [0; 4096];
    if e(syscall::fpath(fd as usize, path)) == !0 {
        !0
    } else {
        e(syscall::chdir(path)) as c_int
    }
}

pub fn fcntl(fd: c_int, cmd: c_int, args: c_int) -> c_int {
    e(syscall::fcntl(fd as usize, cmd as usize, args as usize)) as c_int
}

pub fn fork() -> pid_t {
    e(unsafe { syscall::clone(0) }) as pid_t
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

pub fn mkdir(path: *const c_char, mode: mode_t) -> c_int {
    let flags = O_CREAT | O_EXCL | O_CLOEXEC | O_DIRECTORY | mode as usize & 0o777;
    let path = unsafe { c_str(path) };
    match syscall::open(path, flags) {
        Ok(fd) => {
            syscall::close(fd);
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

pub fn rename(old: *const c_char, new: *const c_char) -> c_int {
    let (old_path, new_path) = unsafe { (c_str(old), c_str(new)) };
    match syscall::open(old_path, O_WRONLY) {
        Ok(fd) => {
            let retval = syscall::frename(fd, new_path);
            let _ = syscall::close(fd);
            e(retval) as c_int
        }
        err => e(err) as c_int,
    }
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
