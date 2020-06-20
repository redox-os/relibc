// Wrapper over the access syscall that doesn't touch errno variable,
// Do not use outside of ld_so

use crate::{c_str::CStr, platform::types::*};

#[cfg(target_os = "redox")]
use crate::header::unistd::{F_OK, R_OK, W_OK, X_OK};

#[cfg(target_os = "linux")]
pub unsafe fn access(path: *const c_char, mode: c_int) -> c_int {
    let path = CStr::from_ptr(path);
    syscall!(ACCESS, (path).as_ptr(), mode) as c_int
}

// Wrapper over the systemcall, Do not use outside of ld_so
#[cfg(target_os = "redox")]
pub unsafe fn access(path: *const c_char, mode: c_int) -> c_int {
    let path = CStr::from_ptr(path).to_bytes();
    let fd = match syscall::open(path, syscall::O_CLOEXEC) {
        Ok(fd) => fd,
        _ => return -1,
    };
    if mode == F_OK {
        return 0;
    }
    let mut stat = syscall::Stat::default();
    if syscall::fstat(fd, &mut stat).is_err() {
        return -1;
    }
    let uid = match syscall::getuid() {
        Ok(uid) => uid,
        Err(_) => return -1,
    };
    let gid = match syscall::getgid() {
        Ok(gid) => gid,
        Err(_) => return -1,
    };

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
        return -1;
    }
    0
}
