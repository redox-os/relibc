//! unistd implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/unistd.h.html

use core::{convert::TryFrom, mem, ptr, slice};

use crate::{
    c_str::CStr,
    header::{
        errno, fcntl, limits, stdlib::getenv, sys_ioctl, sys_time, sys_utsname, termios,
        time::timespec,
    },
    platform::{self, types::*, Pal, Sys},
};
use alloc::collections::LinkedList;

pub use self::{brk::*, getopt::*, pathconf::*, sysconf::*};

mod brk;
mod getopt;
mod pathconf;
mod sysconf;

pub const F_OK: c_int = 0;
pub const R_OK: c_int = 4;
pub const W_OK: c_int = 2;
pub const X_OK: c_int = 1;

pub const SEEK_SET: c_int = 0;
pub const SEEK_CUR: c_int = 1;
pub const SEEK_END: c_int = 2;

pub const F_ULOCK: c_int = 0;
pub const F_LOCK: c_int = 1;
pub const F_TLOCK: c_int = 2;
pub const F_TEST: c_int = 3;

pub const STDIN_FILENO: c_int = 0;
pub const STDOUT_FILENO: c_int = 1;
pub const STDERR_FILENO: c_int = 2;

#[thread_local]
pub static mut fork_hooks_static: Option<[LinkedList<extern "C" fn()>; 3]> = None;

unsafe fn init_fork_hooks<'a>() -> &'a mut [LinkedList<extern "C" fn()>; 3] {
    // Transmute the lifetime so we can return here. Should be safe as
    // long as one does not access the original fork_hooks.
    mem::transmute(
        fork_hooks_static
            .get_or_insert_with(|| [LinkedList::new(), LinkedList::new(), LinkedList::new()]),
    )
}

#[no_mangle]
pub extern "C" fn _exit(status: c_int) {
    Sys::exit(status)
}

#[no_mangle]
pub unsafe extern "C" fn access(path: *const c_char, mode: c_int) -> c_int {
    let path = CStr::from_ptr(path);
    Sys::access(path, mode)
}

#[no_mangle]
pub extern "C" fn alarm(seconds: c_uint) -> c_uint {
    let mut timer = sys_time::itimerval {
        it_value: sys_time::timeval {
            tv_sec: seconds as time_t,
            tv_usec: 0,
        },
        ..Default::default()
    };
    let errno_backup = unsafe { platform::errno };
    let secs = if sys_time::setitimer(sys_time::ITIMER_REAL, &timer, &mut timer) < 0 {
        0
    } else {
        timer.it_value.tv_sec as c_uint + if timer.it_value.tv_usec > 0 { 1 } else { 0 }
    };
    unsafe {
        platform::errno = errno_backup;
    }

    secs
}

#[no_mangle]
pub unsafe extern "C" fn chdir(path: *const c_char) -> c_int {
    let path = CStr::from_ptr(path);
    Sys::chdir(path)
}

#[no_mangle]
pub extern "C" fn chroot(path: *const c_char) -> c_int {
    // TODO: Implement
    unsafe {
        platform::errno = crate::header::errno::EPERM;
    }

    -1
}

#[no_mangle]
pub unsafe extern "C" fn chown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    let path = CStr::from_ptr(path);
    Sys::chown(path, owner, group)
}

#[no_mangle]
pub extern "C" fn close(fildes: c_int) -> c_int {
    Sys::close(fildes)
}

// #[no_mangle]
pub extern "C" fn confstr(name: c_int, buf: *mut c_char, len: size_t) -> size_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn crypt(key: *const c_char, salt: *const c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn dup(fildes: c_int) -> c_int {
    Sys::dup(fildes)
}

#[no_mangle]
pub extern "C" fn dup2(fildes: c_int, fildes2: c_int) -> c_int {
    Sys::dup2(fildes, fildes2)
}

// #[no_mangle]
pub extern "C" fn encrypt(block: [c_char; 64], edflag: c_int) {
    unimplemented!();
}

// #[no_mangle]
// pub extern "C" fn execl(path: *const c_char, args: *const *mut c_char) -> c_int {
//     unimplemented!();
// }

// #[no_mangle]
// pub extern "C" fn execle(
//   path: *const c_char,
//   args: *const *mut c_char,
//   envp: *const *mut c_char,
// ) -> c_int {
//     unimplemented!();
// }

// #[no_mangle]
// pub extern "C" fn execlp(file: *const c_char, args: *const *mut c_char) -> c_int {
//     unimplemented!();
// }

#[no_mangle]
pub unsafe extern "C" fn execv(path: *const c_char, argv: *const *mut c_char) -> c_int {
    execve(path, argv, platform::environ)
}

#[no_mangle]
pub unsafe extern "C" fn execve(
    path: *const c_char,
    argv: *const *mut c_char,
    envp: *const *mut c_char,
) -> c_int {
    let path = CStr::from_ptr(path);
    Sys::execve(path, argv, envp)
}

#[cfg(target_os = "linux")]
const PATH_SEPARATOR: u8 = b':';

#[cfg(target_os = "redox")]
const PATH_SEPARATOR: u8 = b';';

#[no_mangle]
pub unsafe extern "C" fn execvp(file: *const c_char, argv: *const *mut c_char) -> c_int {
    let file = CStr::from_ptr(file);

    if file.to_bytes().contains(&b'/')
        || (cfg!(target_os = "redox") && file.to_bytes().contains(&b':'))
    {
        execv(file.as_ptr(), argv)
    } else {
        let mut error = errno::ENOENT;

        let path_env = getenv(c_str!("PATH\0").as_ptr());
        if !path_env.is_null() {
            let path_env = CStr::from_ptr(path_env);
            for path in path_env.to_bytes().split(|&b| b == PATH_SEPARATOR) {
                let mut program = path.to_vec();
                program.push(b'/');
                program.extend_from_slice(file.to_bytes());
                program.push(b'\0');

                let program_c = CStr::from_bytes_with_nul(&program).unwrap();
                execv(program_c.as_ptr(), argv);

                match platform::errno {
                    errno::ENOENT => (),
                    other => error = other,
                }
            }
        }

        platform::errno = error;
        -1
    }
}

#[no_mangle]
pub extern "C" fn fchown(fildes: c_int, owner: uid_t, group: gid_t) -> c_int {
    Sys::fchown(fildes, owner, group)
}

#[no_mangle]
pub extern "C" fn fchdir(fildes: c_int) -> c_int {
    Sys::fchdir(fildes)
}

// #[no_mangle]
pub extern "C" fn fdatasync(fildes: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fork() -> pid_t {
    let fork_hooks = unsafe { init_fork_hooks() };
    for prepare in &fork_hooks[0] {
        prepare();
    }
    let pid = Sys::fork();
    if pid == 0 {
        for child in &fork_hooks[2] {
            child();
        }
    } else if pid != -1 {
        for parent in &fork_hooks[1] {
            parent();
        }
    }
    pid
}

#[no_mangle]
pub extern "C" fn fsync(fildes: c_int) -> c_int {
    Sys::fsync(fildes)
}

#[no_mangle]
pub extern "C" fn ftruncate(fildes: c_int, length: off_t) -> c_int {
    Sys::ftruncate(fildes, length)
}

#[no_mangle]
pub extern "C" fn getcwd(mut buf: *mut c_char, mut size: size_t) -> *mut c_char {
    let alloc = buf.is_null();
    let mut stack_buf = [0; limits::PATH_MAX];
    if alloc {
        buf = stack_buf.as_mut_ptr();
        size = stack_buf.len();
    }

    let ret = Sys::getcwd(buf, size);
    if ret.is_null() {
        return ptr::null_mut();
    }

    if alloc {
        let len = stack_buf
            .iter()
            .position(|b| *b == 0)
            .expect("no nul-byte in getcwd string")
            + 1;
        let heap_buf = unsafe { platform::alloc(len) as *mut c_char };
        for i in 0..len {
            unsafe {
                *heap_buf.add(i) = stack_buf[i];
            }
        }
        heap_buf
    } else {
        ret
    }
}

// #[no_mangle]
pub extern "C" fn getdtablesize() -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getegid() -> gid_t {
    Sys::getegid()
}

#[no_mangle]
pub extern "C" fn geteuid() -> uid_t {
    Sys::geteuid()
}

#[no_mangle]
pub extern "C" fn getgid() -> gid_t {
    Sys::getgid()
}

// #[no_mangle]
pub extern "C" fn getgroups(gidsetsize: c_int, grouplist: *mut gid_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn gethostid() -> c_long {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn gethostname(mut name: *mut c_char, mut len: size_t) -> c_int {
    let mut uts = mem::MaybeUninit::<sys_utsname::utsname>::uninit();
    let err = Sys::uname(uts.as_mut_ptr());
    if err < 0 {
        mem::forget(uts);
        return err;
    }
    for c in uts.assume_init().nodename.iter() {
        if len == 0 {
            break;
        }
        len -= 1;

        *name = *c;

        if *name == 0 {
            // We do want to copy the zero also, so we check this after the copying.
            break;
        }

        name = name.offset(1);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn getlogin() -> *mut c_char {
    static mut LOGIN: [c_char; 256] = [0; 256];
    if getlogin_r(LOGIN.as_mut_ptr(), LOGIN.len()) == 0 {
        LOGIN.as_mut_ptr()
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn getlogin_r(name: *mut c_char, namesize: size_t) -> c_int {
    //TODO: Determine correct getlogin result on Redox
    unsafe { platform::errno = errno::ENOENT };
    -1
}

#[no_mangle]
pub extern "C" fn getpagesize() -> c_int {
    match c_int::try_from(sysconf(_SC_PAGESIZE)) {
        Ok(page_size) => page_size,
        Err(_) => {
            /* Behavior not specified by POSIX for this case. The -1
             * value mimics sysconf()'s behavior, though.
             *
             * As specified for the limits.h header, the minimum
             * acceptable value for {PAGESIZE} is 1. The -1 value thus
             * cannot be mistaken for an acceptable value.
             *
             * POSIX does not specify any possible errors for this
             * function, hence no errno setting. */
            -1
        }
    }
}

// #[no_mangle]
pub extern "C" fn getpass(prompt: *const c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getpgid(pid: pid_t) -> pid_t {
    Sys::getpgid(pid)
}

#[no_mangle]
pub extern "C" fn getpgrp() -> pid_t {
    Sys::getpgid(Sys::getpid())
}

#[no_mangle]
pub extern "C" fn getpid() -> pid_t {
    Sys::getpid()
}

#[no_mangle]
pub extern "C" fn getppid() -> pid_t {
    Sys::getppid()
}

// #[no_mangle]
pub extern "C" fn getsid(pid: pid_t) -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getuid() -> uid_t {
    Sys::getuid()
}

#[no_mangle]
pub extern "C" fn getwd(path_name: *mut c_char) -> *mut c_char {
    getcwd(path_name, limits::PATH_MAX)
}

#[no_mangle]
pub extern "C" fn isatty(fd: c_int) -> c_int {
    let mut t = termios::termios::default();
    if unsafe { termios::tcgetattr(fd, &mut t as *mut termios::termios) == 0 } {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn lchown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    let path = CStr::from_ptr(path);
    Sys::lchown(path, owner, group)
}

#[no_mangle]
pub unsafe extern "C" fn link(path1: *const c_char, path2: *const c_char) -> c_int {
    let path1 = CStr::from_ptr(path1);
    let path2 = CStr::from_ptr(path2);
    Sys::link(path1, path2)
}

// #[no_mangle]
pub extern "C" fn lockf(fildes: c_int, function: c_int, size: off_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn lseek(fildes: c_int, offset: off_t, whence: c_int) -> off_t {
    Sys::lseek(fildes, offset, whence)
}

// #[no_mangle]
pub extern "C" fn nice(incr: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pause() -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn pipe(fildes: *mut c_int) -> c_int {
    pipe2(fildes, 0)
}

#[no_mangle]
pub unsafe extern "C" fn pipe2(fildes: *mut c_int, flags: c_int) -> c_int {
    Sys::pipe2(slice::from_raw_parts_mut(fildes, 2), flags)
}

#[no_mangle]
pub extern "C" fn pread(fildes: c_int, buf: *mut c_void, nbyte: size_t, offset: off_t) -> ssize_t {
    //TODO: better pread using system calls

    let previous = lseek(fildes, offset, SEEK_SET);
    if previous == -1 {
        return -1;
    }

    let res = read(fildes, buf, nbyte);
    if res < 0 {
        return res;
    }

    if lseek(fildes, previous, SEEK_SET) == -1 {
        return -1;
    }

    res
}

#[no_mangle]
pub extern "C" fn pthread_atfork(
    prepare: Option<extern "C" fn()>,
    parent: Option<extern "C" fn()>,
    child: Option<extern "C" fn()>,
) -> c_int {
    let fork_hooks = unsafe { init_fork_hooks() };
    if let Some(prepare) = prepare {
        fork_hooks[0].push_back(prepare);
    }
    if let Some(parent) = parent {
        fork_hooks[1].push_back(parent);
    }
    if let Some(child) = child {
        fork_hooks[2].push_back(child);
    }
    0
}

#[no_mangle]
pub extern "C" fn pwrite(
    fildes: c_int,
    buf: *const c_void,
    nbyte: size_t,
    offset: off_t,
) -> ssize_t {
    //TODO: better pwrite using system calls

    let previous = lseek(fildes, offset, SEEK_SET);
    if previous == -1 {
        return -1;
    }

    let res = write(fildes, buf, nbyte);
    if res < 0 {
        return res;
    }

    if lseek(fildes, previous, SEEK_SET) == -1 {
        return -1;
    }

    res
}

#[no_mangle]
pub extern "C" fn read(fildes: c_int, buf: *const c_void, nbyte: size_t) -> ssize_t {
    let buf = unsafe { slice::from_raw_parts_mut(buf as *mut u8, nbyte as usize) };
    trace_expr!(
        Sys::read(fildes, buf),
        "read({}, {:p}, {})",
        fildes,
        buf,
        nbyte
    )
}

#[no_mangle]
pub unsafe extern "C" fn readlink(
    path: *const c_char,
    buf: *mut c_char,
    bufsize: size_t,
) -> ssize_t {
    let path = CStr::from_ptr(path);
    let buf = slice::from_raw_parts_mut(buf as *mut u8, bufsize as usize);
    Sys::readlink(path, buf)
}

#[no_mangle]
pub unsafe extern "C" fn rmdir(path: *const c_char) -> c_int {
    let path = CStr::from_ptr(path);
    Sys::rmdir(path)
}

#[no_mangle]
pub extern "C" fn setgid(gid: gid_t) -> c_int {
    Sys::setregid(gid, gid)
}

#[no_mangle]
pub extern "C" fn setpgid(pid: pid_t, pgid: pid_t) -> c_int {
    Sys::setpgid(pid, pgid)
}

#[no_mangle]
pub extern "C" fn setpgrp() -> pid_t {
    setpgid(0, 0)
}

#[no_mangle]
pub extern "C" fn setregid(rgid: gid_t, egid: gid_t) -> c_int {
    Sys::setregid(rgid, egid)
}

#[no_mangle]
pub extern "C" fn setreuid(ruid: uid_t, euid: uid_t) -> c_int {
    Sys::setreuid(ruid, euid)
}

// #[no_mangle]
pub extern "C" fn setsid() -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setuid(uid: uid_t) -> c_int {
    Sys::setreuid(uid, uid)
}

#[no_mangle]
pub extern "C" fn sleep(seconds: c_uint) -> c_uint {
    let rqtp = timespec {
        tv_sec: seconds as i64,
        tv_nsec: 0,
    };
    let rmtp = ptr::null_mut();
    Sys::nanosleep(&rqtp, rmtp);
    0
}

#[no_mangle]
pub extern "C" fn swab(src: *const c_void, dest: *mut c_void, nbytes: ssize_t) {
    if nbytes <= 0 {
        return;
    }
    let number_of_swaps = nbytes / 2;
    let mut offset = 0;
    for i in 0..number_of_swaps {
        unsafe {
            src.offset(offset).copy_to(dest.offset(offset + 1), 1);
            src.offset(offset + 1).copy_to(dest.offset(offset), 1);
        }
        offset += 2;
    }
}

#[no_mangle]
pub unsafe extern "C" fn symlink(path1: *const c_char, path2: *const c_char) -> c_int {
    let path1 = CStr::from_ptr(path1);
    let path2 = CStr::from_ptr(path2);
    Sys::symlink(path1, path2)
}

// #[no_mangle]
pub extern "C" fn sync() {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn tcgetpgrp(fd: c_int) -> pid_t {
    let mut pgrp = 0;
    if unsafe { sys_ioctl::ioctl(fd, sys_ioctl::TIOCGPGRP, &mut pgrp as *mut pid_t as _) } < 0 {
        return -1;
    }
    pgrp
}

#[no_mangle]
pub extern "C" fn tcsetpgrp(fd: c_int, pgrp: pid_t) -> c_int {
    if unsafe { sys_ioctl::ioctl(fd, sys_ioctl::TIOCSPGRP, &pgrp as *const pid_t as _) } < 0 {
        return -1;
    }
    pgrp
}

#[no_mangle]
pub extern "C" fn truncate(path: *const c_char, length: off_t) -> c_int {
    let file = unsafe { CStr::from_ptr(path) };
    let fd = Sys::open(file, fcntl::O_WRONLY, 0);
    if fd < 0 {
        return -1;
    }

    let res = ftruncate(fd, length);

    Sys::close(fd);

    res
}

#[no_mangle]
pub unsafe extern "C" fn ttyname(fildes: c_int) -> *mut c_char {
    static mut TTYNAME: [c_char; 4096] = [0; 4096];
    if ttyname_r(fildes, TTYNAME.as_mut_ptr(), TTYNAME.len()) == 0 {
        TTYNAME.as_mut_ptr()
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn ttyname_r(fildes: c_int, name: *mut c_char, namesize: size_t) -> c_int {
    let name = unsafe { slice::from_raw_parts_mut(name as *mut u8, namesize) };
    if name.is_empty() {
        return errno::ERANGE;
    }

    let len = Sys::fpath(fildes, &mut name[..namesize - 1]);
    if len < 0 {
        return unsafe { -platform::errno };
    }
    name[len as usize] = 0;

    0
}

#[no_mangle]
pub extern "C" fn ualarm(usecs: useconds_t, interval: useconds_t) -> useconds_t {
    let mut timer = sys_time::itimerval {
        it_value: sys_time::timeval {
            tv_sec: 0,
            tv_usec: usecs as suseconds_t,
        },
        it_interval: sys_time::timeval {
            tv_sec: 0,
            tv_usec: interval as suseconds_t,
        },
    };
    let errno_backup = unsafe { platform::errno };
    let ret = if sys_time::setitimer(sys_time::ITIMER_REAL, &timer, &mut timer) < 0 {
        0
    } else {
        timer.it_value.tv_sec as useconds_t * 1_000_000 + timer.it_value.tv_usec as useconds_t
    };
    unsafe {
        platform::errno = errno_backup;
    }

    ret
}

#[no_mangle]
pub unsafe extern "C" fn unlink(path: *const c_char) -> c_int {
    let path = CStr::from_ptr(path);
    Sys::unlink(path)
}

#[no_mangle]
pub extern "C" fn usleep(useconds: useconds_t) -> c_int {
    let rqtp = timespec {
        tv_sec: (useconds / 1_000_000) as i64,
        tv_nsec: ((useconds % 1_000_000) * 1000) as i64,
    };
    let rmtp = ptr::null_mut();
    Sys::nanosleep(&rqtp, rmtp)
}

// #[no_mangle]
pub extern "C" fn vfork() -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn write(fildes: c_int, buf: *const c_void, nbyte: size_t) -> ssize_t {
    let buf = unsafe { slice::from_raw_parts(buf as *const u8, nbyte as usize) };
    Sys::write(fildes, buf)
}
