use crate::{
    header::{
        errno,
        limits::{
            FILESIZEBITS, LINK_MAX, MAX_CANON, MAX_INPUT, NAME_MAX, PATH_MAX, PIPE_BUF,
            POSIX_ALLOC_SIZE_MIN, SYMLINK_MAX,
        },
        unistd::_POSIX_VDISABLE,
    },
    platform::{
        self,
        types::{c_char, c_int, c_long},
    },
};

/// Corresponding pathconf constant for `LINK_MAX` from `limits.h`.
pub const _PC_LINK_MAX: c_int = 0;
/// Corresponding pathconf constant for `MAX_CANON` from `limits.h`.
pub const _PC_MAX_CANON: c_int = 1;
/// Corresponding pathconf constant for `MAX_INPUT` from `limits.h`.
pub const _PC_MAX_INPUT: c_int = 2;
/// Corresponding pathconf constant for `NAME_MAX` from `limits.h`.
pub const _PC_NAME_MAX: c_int = 3;
/// Corresponding pathconf constant for `PATH_MAX` from `limits.h`.
pub const _PC_PATH_MAX: c_int = 4;
/// Corresponding pathconf constant for `PIPE_BUF` from `limits.h`.
pub const _PC_PIPE_BUF: c_int = 5;
/// Corresponding pathconf constant for `_POSIX_CHOWN_RESTRICTED` from
/// `unistd.h`.
pub const _PC_CHOWN_RESTRICTED: c_int = 6;
/// Corresponding pathconf constant for `_POSIX_NO_TRUNC` from `unistd.h`.
pub const _PC_NO_TRUNC: c_int = 7;
/// Corresponding pathconf constant for `_POSIX_VDISABLE` from `unistd.h`.
pub const _PC_VDISABLE: c_int = 8;
/// Corresponding pathconf constant for `_POSIX_SYNC_IO` from `unistd.h`.
pub const _PC_SYNC_IO: c_int = 9;
/// Corresponding pathconf constant for `_POSIX_ASYNC_IO` from `unistd.h`.
pub const _PC_ASYNC_IO: c_int = 10;
/// Corresponding pathconf constant for `_POSIX_PRIO_IO` from `unistd.h`.
pub const _PC_PRIO_IO: c_int = 11;
pub const _PC_SOCK_MAXBUF: c_int = 12;
/// Corresponding pathconf constant for `FILESIZEBITS` from `limits.h`.
pub const _PC_FILESIZEBITS: c_int = 13;
/// Corresponding pathconf constant for `POSIX_REC_INCR_XFER_SIZE` from
/// `limits.h`.
pub const _PC_REC_INCR_XFER_SIZE: c_int = 14;
/// Corresponding pathconf constant for `POSIX_REC_MAX_XFER_SIZE` from
/// `limits.h`.
pub const _PC_REC_MAX_XFER_SIZE: c_int = 15;
/// Corresponding pathconf constant for `POSIX_REC_MIN_XFER_SIZE` from
/// `limits.h`.
pub const _PC_REC_MIN_XFER_SIZE: c_int = 16;
/// Corresponding pathconf constant for `POSIX_REC_XFER_ALIGN` from `limits.h`.
pub const _PC_REC_XFER_ALIGN: c_int = 17;
/// Corresponding pathconf constant for `POSIX_ALLOC_SIZE_MIN` from `limits.h`.
pub const _PC_ALLOC_SIZE_MIN: c_int = 18;
/// Corresponding pathconf constant for `SYMLINK_MAX` from `limits.h`.
pub const _PC_SYMLINK_MAX: c_int = 19;
/// Corresponding pathconf constant for `_POSIX2_SYMLINKS` from `unistd.h`.
pub const _PC_2_SYMLINKS: c_int = 20;

fn pc(name: c_int) -> c_long {
    // Settings from musl, some adjusted
    match name {
        _PC_LINK_MAX => LINK_MAX,
        _PC_MAX_CANON => MAX_CANON,
        _PC_MAX_INPUT => MAX_INPUT,
        _PC_NAME_MAX => NAME_MAX.try_into().unwrap_or(-1),
        _PC_PATH_MAX => PATH_MAX.try_into().unwrap_or(-1),
        _PC_PIPE_BUF => PIPE_BUF,
        _PC_CHOWN_RESTRICTED => 1,
        _PC_NO_TRUNC => 1,
        _PC_VDISABLE => _POSIX_VDISABLE.into(),
        _PC_SYNC_IO => 1,
        _PC_ASYNC_IO => -1,
        _PC_PRIO_IO => -1,
        _PC_SOCK_MAXBUF => -1,
        _PC_FILESIZEBITS => FILESIZEBITS,
        _PC_REC_INCR_XFER_SIZE => -1,
        _PC_REC_MAX_XFER_SIZE => -1,
        _PC_REC_MIN_XFER_SIZE => 4096,
        _PC_REC_XFER_ALIGN => 4096,
        _PC_ALLOC_SIZE_MIN => POSIX_ALLOC_SIZE_MIN,
        _PC_SYMLINK_MAX => SYMLINK_MAX,
        _PC_2_SYMLINKS => 1,
        _ => {
            platform::ERRNO.set(errno::EINVAL);
            -1
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fpathconf.html>.
///
/// Determines the current value of a configurable limit or option (variable)
/// that is associated with a file or directory.
///
/// Upon success, returns the value of the variable corresponding to `name`. If
/// `name` is invalid, returns `-1` and sets errno to indicate the error.
///
/// # Implementation
/// `_fildes` is ignored.
#[unsafe(no_mangle)]
pub extern "C" fn fpathconf(_fildes: c_int, name: c_int) -> c_long {
    pc(name)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fpathconf.html>.
///
/// Determines the current value of a configurable limit or option (variable)
/// that is associated with a file or directory.
///
/// Upon success, returns the value of the variable corresponding to `name`. If
/// `name` is invalid, returns `-1` and sets errno to indicate the error.
///
/// # Implementation
/// `_path` is ignored.
#[unsafe(no_mangle)]
pub extern "C" fn pathconf(_path: *const c_char, name: c_int) -> c_long {
    pc(name)
}
