//! `unistd.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/unistd.h.html>.

use core::{
    convert::TryFrom,
    ffi::VaListImpl,
    mem::{self, MaybeUninit},
    ptr, slice,
};

use crate::{
    c_str::CStr,
    error::{Errno, ResultExt},
    header::{
        crypt::{crypt_data, crypt_r},
        errno::{self, ENAMETOOLONG},
        fcntl, limits,
        stdlib::getenv,
        sys_ioctl, sys_resource,
        sys_select::timeval,
        sys_time, sys_utsname, termios,
        time::timespec,
    },
    out::Out,
    platform::{
        self, ERRNO, Pal, Sys,
        types::{
            c_char, c_int, c_long, c_short, c_uint, c_ulonglong, c_void, gid_t, off_t, pid_t,
            size_t, ssize_t, suseconds_t, time_t, uid_t, useconds_t,
        },
    },
};

pub use self::{brk::*, getopt::*, getpass::getpass, pathconf::*, sysconf::*};
pub use crate::header::pthread::fork_hooks;

// Inclusion of ctermid() prototype marked as obsolescent since Issue 7, cf.
// <https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/unistd.h.html>.
// cuserid() marked legacy in Issue 5.
pub use crate::header::stdio::{ctermid, cuserid};

// TODO: implement and reexport fcntl functions:
//pub use crate::header::fcntl::{faccessat, fchownat, fexecve, linkat, readlinkat, symlinkat, unlinkat};

use super::{
    errno::{E2BIG, EINVAL, ENOMEM},
    stdio::snprintf,
};

mod brk;
mod getopt;
mod getpass;
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

pub const L_cuserid: usize = 9;

// confstr constants
// These are copied from Rust's libc and match musl as well.
pub const _CS_PATH: c_int = 0;
pub const _CS_POSIX_V6_WIDTH_RESTRICTED_ENVS: c_int = 1;
pub const _CS_POSIX_V5_WIDTH_RESTRICTED_ENVS: c_int = 4;
pub const _CS_POSIX_V7_WIDTH_RESTRICTED_ENVS: c_int = 5;
pub const _CS_POSIX_V6_ILP32_OFF32_CFLAGS: c_int = 1116;
pub const _CS_POSIX_V6_ILP32_OFF32_LDFLAGS: c_int = 1117;
pub const _CS_POSIX_V6_ILP32_OFF32_LIBS: c_int = 1118;
pub const _CS_POSIX_V6_ILP32_OFF32_LINTFLAGS: c_int = 1119;
pub const _CS_POSIX_V6_ILP32_OFFBIG_CFLAGS: c_int = 1120;
pub const _CS_POSIX_V6_ILP32_OFFBIG_LDFLAGS: c_int = 1121;
pub const _CS_POSIX_V6_ILP32_OFFBIG_LIBS: c_int = 1122;
pub const _CS_POSIX_V6_ILP32_OFFBIG_LINTFLAGS: c_int = 1123;
pub const _CS_POSIX_V6_LP64_OFF64_CFLAGS: c_int = 1124;
pub const _CS_POSIX_V6_LP64_OFF64_LDFLAGS: c_int = 1125;
pub const _CS_POSIX_V6_LP64_OFF64_LIBS: c_int = 1126;
pub const _CS_POSIX_V6_LP64_OFF64_LINTFLAGS: c_int = 1127;
pub const _CS_POSIX_V6_LPBIG_OFFBIG_CFLAGS: c_int = 1128;
pub const _CS_POSIX_V6_LPBIG_OFFBIG_LDFLAGS: c_int = 1129;
pub const _CS_POSIX_V6_LPBIG_OFFBIG_LIBS: c_int = 1130;
pub const _CS_POSIX_V6_LPBIG_OFFBIG_LINTFLAGS: c_int = 1131;
pub const _CS_POSIX_V7_ILP32_OFF32_CFLAGS: c_int = 1132;
pub const _CS_POSIX_V7_ILP32_OFF32_LDFLAGS: c_int = 1133;
pub const _CS_POSIX_V7_ILP32_OFF32_LIBS: c_int = 1134;
pub const _CS_POSIX_V7_ILP32_OFF32_LINTFLAGS: c_int = 1135;
pub const _CS_POSIX_V7_ILP32_OFFBIG_CFLAGS: c_int = 1136;
pub const _CS_POSIX_V7_ILP32_OFFBIG_LDFLAGS: c_int = 1137;
pub const _CS_POSIX_V7_ILP32_OFFBIG_LIBS: c_int = 1138;
pub const _CS_POSIX_V7_ILP32_OFFBIG_LINTFLAGS: c_int = 1139;
pub const _CS_POSIX_V7_LP64_OFF64_CFLAGS: c_int = 1140;
pub const _CS_POSIX_V7_LP64_OFF64_LDFLAGS: c_int = 1141;
pub const _CS_POSIX_V7_LP64_OFF64_LIBS: c_int = 1142;
pub const _CS_POSIX_V7_LP64_OFF64_LINTFLAGS: c_int = 1143;
pub const _CS_POSIX_V7_LPBIG_OFFBIG_CFLAGS: c_int = 1144;
pub const _CS_POSIX_V7_LPBIG_OFFBIG_LDFLAGS: c_int = 1145;
pub const _CS_POSIX_V7_LPBIG_OFFBIG_LIBS: c_int = 1146;
pub const _CS_POSIX_V7_LPBIG_OFFBIG_LINTFLAGS: c_int = 1147;

// Re-exported from pthread.h. `pthread_atfork` should be in pthread.h according to the
// standard, but glibc exports it here as well. We ONLY exported it in unistd.h till recently.
unsafe extern "C" {
    #[unsafe(no_mangle)]
    pub fn pthread_atfork(
        prepare: Option<extern "C" fn()>,
        parent: Option<extern "C" fn()>,
        child: Option<extern "C" fn()>,
    ) -> c_int;
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fork.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _Fork() -> pid_t {
    unsafe { Sys::fork() }.or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/_Exit.html>.
#[unsafe(no_mangle)]
pub extern "C" fn _exit(status: c_int) -> ! {
    Sys::exit(status)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/access.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn access(path: *const c_char, mode: c_int) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    Sys::access(path, mode).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/alarm.html>.
#[unsafe(no_mangle)]
pub extern "C" fn alarm(seconds: c_uint) -> c_uint {
    // TODO setitimer is unimplemented on Redox and obsolete
    let timer = sys_time::itimerval {
        it_value: timeval {
            tv_sec: seconds as time_t,
            tv_usec: 0,
        },
        ..Default::default()
    };
    let mut otimer = sys_time::itimerval::default();

    let errno_backup = platform::ERRNO.get();
    let secs = if unsafe { sys_time::setitimer(sys_time::ITIMER_REAL, &timer, &mut otimer) } < 0 {
        0
    } else {
        otimer.it_value.tv_sec as c_uint + if otimer.it_value.tv_usec > 0 { 1 } else { 0 }
    };
    platform::ERRNO.set(errno_backup);

    secs
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/chdir.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chdir(path: *const c_char) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    Sys::chdir(path).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/chown.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    Sys::chown(path, owner, group)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/7908799/xsh/chroot.html>.
///
/// # Deprecation
/// The `chroot()` function was marked legacy in the System Interface & Headers
/// Issue 5, and removed in Issue 6.
#[deprecated]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chroot(path: *const c_char) -> c_int {
    // TODO: Implement
    platform::ERRNO.set(crate::header::errno::EPERM);

    -1
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/close.html>.
#[unsafe(no_mangle)]
pub extern "C" fn close(fildes: c_int) -> c_int {
    Sys::close(fildes).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/confstr.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn confstr(name: c_int, buf: *mut c_char, len: size_t) -> size_t {
    // confstr returns the number of bytes required to hold the string INCLUDING the NUL
    // terminator. This is different from other C functions hence the + 1.
    match name {
        _CS_PATH => {
            let posix2_path = c"/usr/bin";
            unsafe { snprintf(buf, len, c"%s".as_ptr(), posix2_path.as_ptr()) + 1 }
                .try_into()
                .unwrap_or_default()
        }
        _CS_POSIX_V6_WIDTH_RESTRICTED_ENVS
        | _CS_POSIX_V5_WIDTH_RESTRICTED_ENVS
        | _CS_POSIX_V7_WIDTH_RESTRICTED_ENVS
        | _CS_POSIX_V6_LP64_OFF64_LIBS..=_CS_POSIX_V7_LPBIG_OFFBIG_LINTFLAGS => 1,
        _ => {
            platform::ERRNO.set(errno::EINVAL);
            0
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/crypt.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn crypt(key: *const c_char, salt: *const c_char) -> *mut c_char {
    let mut data = crypt_data::new();
    unsafe { crypt_r(key, salt, &mut data as *mut _) }
}

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/daemon.3.html>.
#[unsafe(no_mangle)]
pub extern "C" fn daemon(nochdir: c_int, noclose: c_int) -> c_int {
    if nochdir == 0 {
        if Sys::chdir(c"/".into()).map(|()| 0).or_minus_one_errno() < 0 {
            return -1;
        }
    }

    if noclose == 0 {
        let fd = Sys::open(c"/dev/null".into(), fcntl::O_RDWR, 0).or_minus_one_errno();
        if fd < 0 {
            return -1;
        }
        if dup2(fd, 0) < 0 || dup2(fd, 1) < 0 || dup2(fd, 2) < 0 {
            close(fd);
            return -1;
        }
        if fd > 2 {
            close(fd);
        }
    }

    match unsafe { fork() } {
        0 => {}
        -1 => return -1,
        _ => _exit(0),
    }

    if setsid() < 0 {
        return -1;
    }

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/dup.html>.
#[unsafe(no_mangle)]
pub extern "C" fn dup(fildes: c_int) -> c_int {
    Sys::dup(fildes).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/dup.html>.
#[unsafe(no_mangle)]
pub extern "C" fn dup2(fildes: c_int, fildes2: c_int) -> c_int {
    Sys::dup2(fildes, fildes2).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/dup.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn dup3(fildes: c_int, fildes2: c_int, flag: c_int) -> c_int {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/encrypt.html>.
///
/// # Deprecation
/// The `encrypt()` function was marked obsolescent in the Open Group Base Specifications Issue 8.
#[deprecated]
// #[unsafe(no_mangle)]
pub extern "C" fn encrypt(block: [c_char; 64], edflag: c_int) {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/exec.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn execl(
    path: *const c_char,
    arg0: *const c_char,
    mut __valist: ...
) -> c_int {
    unsafe {
        with_argv(__valist, arg0, |args, _remaining_va| {
            execv(path, args.as_ptr().cast())
        })
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/exec.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn execle(
    path: *const c_char,
    arg0: *const c_char,
    mut __valist: ...
) -> c_int {
    unsafe {
        with_argv(__valist, arg0, |args, mut remaining_va| {
            let envp = remaining_va.arg::<*const *mut c_char>();
            execve(path, args.as_ptr().cast(), envp)
        })
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/exec.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn execlp(
    file: *const c_char,
    arg0: *const c_char,
    mut __valist: ...
) -> c_int {
    unsafe {
        with_argv(__valist, arg0, |args, _remaining_va| {
            execvp(file, args.as_ptr().cast())
        })
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/exec.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn execv(path: *const c_char, argv: *const *mut c_char) -> c_int {
    unsafe { execve(path, argv, platform::environ) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/exec.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn execve(
    path: *const c_char,
    argv: *const *mut c_char,
    envp: *const *mut c_char,
) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    unsafe { Sys::execve(path, argv, envp) }
        .map(|()| unreachable!())
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/exec.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fexecve(
    fd: c_int,
    argv: *const *mut c_char,
    envp: *const *mut c_char,
) -> c_int {
    unsafe { Sys::fexecve(fd, argv, envp) }
        .map(|()| unreachable!())
        .or_minus_one_errno()
}

const PATH_SEPARATOR: u8 = b':';

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/exec.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn execvp(file: *const c_char, argv: *const *mut c_char) -> c_int {
    let file = unsafe { CStr::from_ptr(file) };

    if file.to_bytes().contains(&b'/')
        || (cfg!(target_os = "redox") && file.to_bytes().contains(&b':'))
    {
        unsafe { execv(file.as_ptr(), argv) }
    } else {
        let mut error = errno::ENOENT;

        let path_env = unsafe { getenv(c"PATH".as_ptr()) };
        if !path_env.is_null() {
            let path_env = unsafe { CStr::from_ptr(path_env) };
            for path in path_env.to_bytes().split(|&b| b == PATH_SEPARATOR) {
                let file = file.to_bytes();
                let length = file.len() + path.len() + 2;
                let mut program = alloc::vec::Vec::with_capacity(length);
                program.extend_from_slice(path);
                program.push(b'/');
                program.extend_from_slice(file);
                program.push(b'\0');

                let program_c = CStr::from_bytes_with_nul(&program).unwrap();
                unsafe { execv(program_c.as_ptr(), argv) };

                match platform::ERRNO.get() {
                    errno::ENOENT => (),
                    other => error = other,
                }
            }
        }

        platform::ERRNO.set(error);
        -1
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fchdir.html>.
#[unsafe(no_mangle)]
pub extern "C" fn fchdir(fildes: c_int) -> c_int {
    Sys::fchdir(fildes).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fchown.html>.
#[unsafe(no_mangle)]
pub extern "C" fn fchown(fildes: c_int, owner: uid_t, group: gid_t) -> c_int {
    Sys::fchown(fildes, owner, group)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fdatasync.html>.
#[unsafe(no_mangle)]
pub extern "C" fn fdatasync(fildes: c_int) -> c_int {
    Sys::fdatasync(fildes).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fork.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fork() -> pid_t {
    for prepare in unsafe { &fork_hooks[0] } {
        prepare();
    }
    let pid = unsafe { Sys::fork() }.or_minus_one_errno();
    if pid == 0 {
        for child in unsafe { &fork_hooks[2] } {
            child();
        }
    } else if pid != -1 {
        for parent in unsafe { &fork_hooks[1] } {
            parent();
        }
    }
    pid
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fsync.html>.
#[unsafe(no_mangle)]
pub extern "C" fn fsync(fildes: c_int) -> c_int {
    Sys::fsync(fildes).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ftruncate.html>.
#[unsafe(no_mangle)]
pub extern "C" fn ftruncate(fildes: c_int, length: off_t) -> c_int {
    Sys::ftruncate(fildes, length)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getcwd.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getcwd(mut buf: *mut c_char, mut size: size_t) -> *mut c_char {
    let alloc = buf.is_null();
    let mut stack_buf = [0; limits::PATH_MAX];
    if alloc {
        buf = stack_buf.as_mut_ptr();
        size = stack_buf.len();
    }

    let ret = match Sys::getcwd(unsafe { Out::from_raw_parts(buf.cast(), size) }) {
        Ok(()) => buf,
        Err(Errno(errno)) => {
            ERRNO.set(errno);
            return ptr::null_mut();
        }
    };

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

/// See <https://pubs.opengroup.org/onlinepubs/7908799/xsh/getdtablesize.html>.
///
/// # Deprecation
/// The `getdtablesize()` function was marked legacy in the System Interface &
/// Headers Issue 5, and removed in Issue 6.
#[deprecated]
#[unsafe(no_mangle)]
pub extern "C" fn getdtablesize() -> c_int {
    let mut lim = mem::MaybeUninit::<sys_resource::rlimit>::uninit();
    let r = unsafe {
        sys_resource::getrlimit(
            sys_resource::RLIMIT_NOFILE as c_int,
            lim.as_mut_ptr() as *mut sys_resource::rlimit,
        )
    };
    if r == 0 {
        let cur = unsafe { lim.assume_init() }.rlim_cur;
        match cur {
            c if c < i32::MAX as u64 => c as i32,
            _ => i32::MAX,
        };
    }
    -1
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getegid.html>.
#[unsafe(no_mangle)]
pub extern "C" fn getegid() -> gid_t {
    Sys::getegid()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getentropy.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn getentropy(buffer: *mut c_void, length: size_t) -> c_int {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/geteuid.html>.
#[unsafe(no_mangle)]
pub extern "C" fn geteuid() -> uid_t {
    Sys::geteuid()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getgid.html>.
#[unsafe(no_mangle)]
pub extern "C" fn getgid() -> gid_t {
    Sys::getgid()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getgroups.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getgroups(size: c_int, list: *mut gid_t) -> c_int {
    (|| {
        let size = usize::try_from(size)
            // fails for negative size, but EINVAL required if size != 0 && size < actual size,
            // where the actual number of entries in the group list is obviously nonnegative
            .map_err(|_| Errno(EINVAL))?;

        let list = unsafe { Out::from_raw_parts(list, size) };

        Sys::getgroups(list)
    })()
    .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/gethostid.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn gethostid() -> c_long {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/gethostname.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gethostname(mut name: *mut c_char, mut len: size_t) -> c_int {
    let mut uts = mem::MaybeUninit::<sys_utsname::utsname>::uninit();
    // TODO
    let err = Sys::uname(Out::from_uninit_mut(&mut uts))
        .map(|()| 0)
        .or_minus_one_errno();
    if err < 0 {
        mem::forget(uts);
        return err;
    }
    for c in unsafe { uts.assume_init() }.nodename.iter() {
        if len == 0 {
            break;
        }
        len -= 1;

        unsafe { *name = *c };

        if unsafe { *name } == 0 {
            // We do want to copy the zero also, so we check this after the copying.
            break;
        }

        name = unsafe { name.offset(1) };
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getlogin.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getlogin() -> *mut c_char {
    const LOGIN_LEN: usize = 256;
    static mut LOGIN: [c_char; LOGIN_LEN] = [0; LOGIN_LEN];
    if getlogin_r(&raw mut LOGIN as *mut _, LOGIN_LEN) == 0 {
        &raw mut LOGIN as *mut _
    } else {
        ptr::null_mut()
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getlogin.html>.
#[unsafe(no_mangle)]
pub extern "C" fn getlogin_r(name: *mut c_char, namesize: size_t) -> c_int {
    //TODO: Determine correct getlogin result on Redox
    platform::ERRNO.set(errno::ENOENT);
    -1
}

/// See <https://pubs.opengroup.org/onlinepubs/7908799/xsh/getpagesize.html>.
///
/// # Deprecation
/// The `getpagesize()` function was marked legacy in the System Interface &
/// Headers Issue 5, and removed in Issue 6.
#[deprecated]
#[unsafe(no_mangle)]
pub extern "C" fn getpagesize() -> c_int {
    // Panic if we can't uphold the required behavior (no errors are specified for this function)
    Sys::getpagesize()
        .try_into()
        .expect("page size not representable as type `int`")
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getpgid.html>.
#[unsafe(no_mangle)]
pub extern "C" fn getpgid(pid: pid_t) -> pid_t {
    Sys::getpgid(pid).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getpgrp.html>.
#[unsafe(no_mangle)]
pub extern "C" fn getpgrp() -> pid_t {
    Sys::getpgid(Sys::getpid()).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getpid.html>.
#[unsafe(no_mangle)]
pub extern "C" fn getpid() -> pid_t {
    Sys::getpid()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getppid.html>.
#[unsafe(no_mangle)]
pub extern "C" fn getppid() -> pid_t {
    Sys::getppid()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getresgid.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getresgid(rgid: *mut gid_t, egid: *mut gid_t, sgid: *mut gid_t) -> c_int {
    Sys::getresgid(
        unsafe { Out::nullable(rgid) },
        unsafe { Out::nullable(egid) },
        unsafe { Out::nullable(sgid) },
    )
    .map(|()| 0)
    .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getresuid.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getresuid(ruid: *mut uid_t, euid: *mut uid_t, suid: *mut uid_t) -> c_int {
    Sys::getresuid(
        unsafe { Out::nullable(ruid) },
        unsafe { Out::nullable(euid) },
        unsafe { Out::nullable(suid) },
    )
    .map(|()| 0)
    .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getsid.html>.
#[unsafe(no_mangle)]
pub extern "C" fn getsid(pid: pid_t) -> pid_t {
    Sys::getsid(pid).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getuid.html>.
#[unsafe(no_mangle)]
pub extern "C" fn getuid() -> uid_t {
    Sys::getuid()
}

/// See <https://pubs.opengroup.org/onlinepubs/009695399/functions/getwd.html>.
///
/// # Deprecation
/// The `getwd()` function was marked legacy in the Open Group Base
/// Specifications Issue 6, and removed in Issue 7.
#[deprecated]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getwd(path_name: *mut c_char) -> *mut c_char {
    unsafe { getcwd(path_name, limits::PATH_MAX) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/isatty.html>.
#[unsafe(no_mangle)]
pub extern "C" fn isatty(fd: c_int) -> c_int {
    let mut t = termios::termios::default();
    if unsafe { termios::tcgetattr(fd, &mut t as *mut termios::termios) == 0 } {
        1
    } else {
        0
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/lchown.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lchown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    Sys::lchown(path, owner, group)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/link.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn link(path1: *const c_char, path2: *const c_char) -> c_int {
    let path1 = unsafe { CStr::from_ptr(path1) };
    let path2 = unsafe { CStr::from_ptr(path2) };
    Sys::link(path1, path2).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/lockf.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lockf(fildes: c_int, function: c_int, size: off_t) -> c_int {
    let mut fl = fcntl::flock {
        l_type: fcntl::F_WRLCK as c_short,
        l_whence: SEEK_CUR as c_short,
        l_start: 0,
        l_len: size,
        l_pid: -1,
    };

    match function {
        fcntl::F_TEST => {
            fl.l_type = fcntl::F_RDLCK as c_short;
            if unsafe { fcntl::fcntl(fildes, fcntl::F_GETLK, &mut fl as *mut _ as c_ulonglong) } < 0
            {
                return -1;
            }
            if fl.l_type == fcntl::F_UNLCK as c_short || fl.l_pid == getpid() {
                return 0;
            }
            platform::ERRNO.set(errno::EACCES);
            return -1;
        }
        fcntl::F_ULOCK => {
            fl.l_type = fcntl::F_UNLCK as c_short;
            return unsafe {
                fcntl::fcntl(fildes, fcntl::F_SETLK, &mut fl as *mut _ as c_ulonglong)
            };
        }
        fcntl::F_TLOCK => {
            return unsafe {
                fcntl::fcntl(fildes, fcntl::F_SETLK, &mut fl as *mut _ as c_ulonglong)
            };
        }
        fcntl::F_LOCK => {
            return unsafe {
                fcntl::fcntl(fildes, fcntl::F_SETLKW, &mut fl as *mut _ as c_ulonglong)
            };
        }
        _ => {
            platform::ERRNO.set(errno::EINVAL);
            return -1;
        }
    };
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/lseek.html>.
#[unsafe(no_mangle)]
pub extern "C" fn lseek(fildes: c_int, offset: off_t, whence: c_int) -> off_t {
    Sys::lseek(fildes, offset, whence).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/nice.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn nice(incr: c_int) -> c_int {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pause.html>.
// #[unsafe(no_mangle)]
pub extern "C" fn pause() -> c_int {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pipe.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pipe(fildes: *mut c_int) -> c_int {
    unsafe { pipe2(fildes, 0) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pipe.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pipe2(fildes: *mut c_int, flags: c_int) -> c_int {
    Sys::pipe2(unsafe { Out::nonnull(fildes.cast::<[c_int; 2]>()) }, flags)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_close.html>.
#[unsafe(no_mangle)]
pub extern "C" fn posix_close(fildes: c_int, flag: c_int) -> c_int {
    // Since we do not define `POSIX_CLOSE_RESTART`, this function is
    // equivalent to `close`. In the future when we move file descriptors
    // to userspace, it would only make sense to define `POSIX_CLOSE_RESTART`
    // if `close` is not atomic.
    close(fildes)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/read.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pread(
    fildes: c_int,
    buf: *mut c_void,
    nbyte: size_t,
    offset: off_t,
) -> ssize_t {
    Sys::pread(
        fildes,
        unsafe { slice::from_raw_parts_mut(buf.cast::<u8>(), nbyte) },
        offset,
    )
    .map(|read| read as ssize_t)
    .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/write.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pwrite(
    fildes: c_int,
    buf: *const c_void,
    nbyte: size_t,
    offset: off_t,
) -> ssize_t {
    Sys::pwrite(
        fildes,
        unsafe { slice::from_raw_parts(buf.cast::<u8>(), nbyte) },
        offset,
    )
    .map(|read| read as ssize_t)
    .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/read.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn read(fildes: c_int, buf: *mut c_void, nbyte: size_t) -> ssize_t {
    let buf = unsafe { slice::from_raw_parts_mut(buf as *mut u8, nbyte as usize) };
    trace_expr!(
        Sys::read(fildes, buf)
            .map(|read| read as ssize_t)
            .or_minus_one_errno(),
        "read({}, {:p}, {})",
        fildes,
        buf,
        nbyte
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/readlink.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn readlink(
    path: *const c_char,
    buf: *mut c_char,
    bufsize: size_t,
) -> ssize_t {
    let path = unsafe { CStr::from_ptr(path) };
    let buf = unsafe { slice::from_raw_parts_mut(buf as *mut u8, bufsize as usize) };
    Sys::readlink(path, buf)
        .map(|read| read as ssize_t)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/readlink.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn readlinkat(
    dirfd: c_int,
    pathname: *const c_char,
    buf: *mut c_char,
    len: size_t,
) -> ssize_t {
    let pathname = unsafe { CStr::from_ptr(pathname) };
    let mut buf = unsafe { slice::from_raw_parts_mut(buf.cast(), len) };
    Sys::readlinkat(dirfd, pathname, &mut buf)
        .map(|read| {
            read.try_into()
                .map_err(|_| Errno(ENAMETOOLONG))
                .or_minus_one_errno()
        })
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/rmdir.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmdir(path: *const c_char) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    Sys::rmdir(path).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setegid.html>.
#[unsafe(no_mangle)]
pub extern "C" fn setegid(gid: gid_t) -> c_int {
    Sys::setresgid(-1, gid, -1).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/seteuid.html>.
#[unsafe(no_mangle)]
pub extern "C" fn seteuid(uid: uid_t) -> c_int {
    Sys::setresuid(-1, uid, -1).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setgid.html>.
#[unsafe(no_mangle)]
pub extern "C" fn setgid(gid: gid_t) -> c_int {
    Sys::setresgid(gid, gid, -1)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/setgroups.2.html>.
///
/// TODO: specified in `grp.h`?
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setgroups(size: size_t, list: *const gid_t) -> c_int {
    unsafe { Sys::setgroups(size, list) }
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setpgid.html>.
#[unsafe(no_mangle)]
pub extern "C" fn setpgid(pid: pid_t, pgid: pid_t) -> c_int {
    Sys::setpgid(pid, pgid).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/setpgrp.html>.
///
/// # Deprecation
/// The `setpgrp()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 7, and removed in Issue 8.
#[deprecated]
#[unsafe(no_mangle)]
pub extern "C" fn setpgrp() -> pid_t {
    setpgid(0, 0)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setregid.html>.
#[unsafe(no_mangle)]
pub extern "C" fn setregid(rgid: gid_t, egid: gid_t) -> c_int {
    Sys::setresgid(rgid, egid, -1)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setresgid.html>.
#[unsafe(no_mangle)]
pub extern "C" fn setresgid(rgid: gid_t, egid: gid_t, sgid: gid_t) -> c_int {
    Sys::setresgid(rgid, egid, sgid)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setresuid.html>.
#[unsafe(no_mangle)]
pub extern "C" fn setresuid(ruid: uid_t, euid: uid_t, suid: uid_t) -> c_int {
    Sys::setresuid(ruid, euid, suid)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setreuid.html>.
#[unsafe(no_mangle)]
pub extern "C" fn setreuid(ruid: uid_t, euid: uid_t) -> c_int {
    Sys::setresuid(ruid, euid, -1)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setsid.html>.
#[unsafe(no_mangle)]
pub extern "C" fn setsid() -> pid_t {
    Sys::setsid().or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setuid.html>.
#[unsafe(no_mangle)]
pub extern "C" fn setuid(uid: uid_t) -> c_int {
    Sys::setresuid(uid, uid, -1)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sleep.html>.
#[unsafe(no_mangle)]
pub extern "C" fn sleep(seconds: c_uint) -> c_uint {
    let rqtp = timespec {
        tv_sec: seconds as time_t,
        tv_nsec: 0,
    };
    let mut rmtp = timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    // If sleep() returns because the requested time has elapsed, the value returned shall be 0.
    // If sleep() returns due to delivery of a signal, the return value shall be the "unslept" amount
    // (the requested time minus the time actually slept) in seconds.
    match unsafe { Sys::nanosleep(&rqtp, &mut rmtp) } {
        Err(Errno(EINTR)) => rmtp.tv_sec as c_uint,
        r => 0,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/swab.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn swab(src: *const c_void, dest: *mut c_void, nbytes: ssize_t) {
    if nbytes <= 0 {
        return;
    }
    let number_of_swaps = nbytes / 2;
    let mut offset = 0;
    for _ in 0..number_of_swaps {
        unsafe {
            src.offset(offset).copy_to(dest.offset(offset + 1), 1);
            src.offset(offset + 1).copy_to(dest.offset(offset), 1);
        }
        offset += 2;
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/symlink.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn symlink(path1: *const c_char, path2: *const c_char) -> c_int {
    let path1 = unsafe { CStr::from_ptr(path1) };
    let path2 = unsafe { CStr::from_ptr(path2) };
    Sys::symlink(path1, path2).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sync.html>.
#[unsafe(no_mangle)]
pub extern "C" fn sync() {
    Sys::sync();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcgetpgrp.html>.
#[unsafe(no_mangle)]
pub extern "C" fn tcgetpgrp(fd: c_int) -> pid_t {
    let mut pgrp = 0;
    if unsafe { sys_ioctl::ioctl(fd, sys_ioctl::TIOCGPGRP, &mut pgrp as *mut pid_t as _) } < 0 {
        return -1;
    }
    pgrp
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tcsetpgrp.html>.
#[unsafe(no_mangle)]
pub extern "C" fn tcsetpgrp(fd: c_int, pgrp: pid_t) -> c_int {
    if unsafe { sys_ioctl::ioctl(fd, sys_ioctl::TIOCSPGRP, &pgrp as *const pid_t as _) } < 0 {
        return -1;
    }
    pgrp
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/truncate.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn truncate(path: *const c_char, length: off_t) -> c_int {
    let file = unsafe { CStr::from_ptr(path) };
    // TODO: Rustify
    let fd = Sys::open(file, fcntl::O_WRONLY, 0).or_minus_one_errno();
    if fd < 0 {
        return -1;
    }

    let res = ftruncate(fd, length);

    Sys::close(fd);

    res
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ttyname.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ttyname(fildes: c_int) -> *mut c_char {
    const TTYNAME_LEN: usize = 4096;
    static mut TTYNAME: [c_char; TTYNAME_LEN] = [0; TTYNAME_LEN];
    if ttyname_r(fildes, &raw mut TTYNAME as *mut _, TTYNAME_LEN) == 0 {
        &raw mut TTYNAME as *mut _
    } else {
        ptr::null_mut()
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ttyname.html>.
#[unsafe(no_mangle)]
pub extern "C" fn ttyname_r(fildes: c_int, name: *mut c_char, namesize: size_t) -> c_int {
    let name = unsafe { slice::from_raw_parts_mut(name as *mut u8, namesize) };
    if name.is_empty() {
        return errno::ERANGE;
    }

    let len = Sys::fpath(fildes, &mut name[..namesize - 1])
        .map(|read| read as ssize_t)
        .or_minus_one_errno();
    if len < 0 {
        return -platform::ERRNO.get();
    }
    name[len as usize] = 0;

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/009695399/functions/ualarm.html>.
///
/// # Deprecation
/// The `ualarm()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 6, and removed in Issue 7.
#[deprecated]
#[unsafe(no_mangle)]
pub extern "C" fn ualarm(usecs: useconds_t, interval: useconds_t) -> useconds_t {
    // TODO setitimer is unimplemented on Redox and obsolete
    let mut timer = sys_time::itimerval {
        it_value: timeval {
            tv_sec: 0,
            tv_usec: usecs as suseconds_t,
        },
        it_interval: timeval {
            tv_sec: 0,
            tv_usec: interval as suseconds_t,
        },
    };
    let errno_backup = platform::ERRNO.get();
    let ret = if unsafe { sys_time::setitimer(sys_time::ITIMER_REAL, &timer, &mut timer) } < 0 {
        0
    } else {
        timer.it_value.tv_sec as useconds_t * 1_000_000 + timer.it_value.tv_usec as useconds_t
    };
    platform::ERRNO.set(errno_backup);

    ret
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/unlink.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn unlink(path: *const c_char) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    Sys::unlink(path).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/009695399/functions/usleep.html>.
///
/// # Deprecation
/// The `usleep()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 6, and removed in Issue 7.
#[deprecated]
#[unsafe(no_mangle)]
pub extern "C" fn usleep(useconds: useconds_t) -> c_int {
    let rqtp = timespec {
        tv_sec: (useconds / 1_000_000) as time_t,
        tv_nsec: ((useconds % 1_000_000) * 1000) as c_long,
    };
    let rmtp = ptr::null_mut();
    unsafe { Sys::nanosleep(&rqtp, rmtp) }
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/009695399/functions/vfork.html>.
///
/// # Deprecation
/// The `vfork()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 6, and removed in Issue 7.
#[deprecated]
// #[unsafe(no_mangle)]
pub extern "C" fn vfork() -> pid_t {
    unimplemented!();
}

unsafe fn with_argv(
    mut va: VaListImpl,
    arg0: *const c_char,
    f: impl FnOnce(&[*const c_char], VaListImpl) -> c_int,
) -> c_int {
    let argc = 1 + unsafe {
        va.with_copy(|mut copy| {
            core::iter::from_fn(|| Some(copy.arg::<*const c_char>()))
                .position(|p| p.is_null())
                .unwrap()
        })
    };

    let mut stack: [MaybeUninit<*const c_char>; 32] = [MaybeUninit::uninit(); 32];

    let out = if argc < 32 {
        stack.as_mut_slice()
    } else if argc < 4096 {
        // TODO: Use ARG_MAX, not this hardcoded constant
        let ptr =
            unsafe { crate::header::stdlib::malloc((argc + 1) * mem::size_of::<*const c_char>()) };
        if ptr.is_null() {
            platform::ERRNO.set(ENOMEM);
            return -1;
        }
        unsafe { slice::from_raw_parts_mut(ptr.cast::<MaybeUninit<*const c_char>>(), argc + 1) }
    } else {
        platform::ERRNO.set(E2BIG);
        return -1;
    };
    out[0].write(arg0);

    for i in 1..argc {
        out[i].write(unsafe { va.arg::<*const c_char>() });
    }
    out[argc].write(core::ptr::null());
    // NULL
    unsafe { va.arg::<*const c_char>() };

    f(unsafe { (&*out).assume_init_ref() }, va);

    // f only returns if it fails
    if argc >= 32 {
        unsafe { crate::header::stdlib::free(out.as_mut_ptr().cast()) };
    }
    -1
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/write.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn write(fildes: c_int, buf: *const c_void, nbyte: size_t) -> ssize_t {
    let buf = unsafe { slice::from_raw_parts(buf as *const u8, nbyte as usize) };
    Sys::write(fildes, buf)
        .map(|bytes| bytes as ssize_t)
        .or_minus_one_errno()
}
