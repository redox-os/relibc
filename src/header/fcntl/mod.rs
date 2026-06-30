//! `fcntl.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/fcntl.h.html>.

use core::num::NonZeroU64;

use crate::{
    c_str::CStr,
    error::ResultExt,
    platform::{
        Pal, Sys,
        types::{c_char, c_int, c_short, c_ulonglong, mode_t, off_t, pid_t},
    },
};

pub use self::sys::*;
pub use crate::header::bits_open_flags::*;

use super::errno::EINVAL;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

/// Duplicate file descriptor.
pub const F_DUPFD: c_int = 0;
/// Get file descriptor flags.
pub const F_GETFD: c_int = 1;
/// Set file descriptor flags.
pub const F_SETFD: c_int = 2;
/// Get file status flags and file access modes.
pub const F_GETFL: c_int = 3;
/// Set file status flags.
pub const F_SETFL: c_int = 4;
/// Get information about file locks.
pub const F_GETLK: c_int = 5;
/// Set a process-owned file lock.
pub const F_SETLK: c_int = 6;
/// Set a process-owned file lock; wait if blocked.
pub const F_SETLKW: c_int = 7;
/// Get information about file locks.
pub const F_OFD_GETLK: c_int = 36;
/// Set an OFD-owned file lock.
pub const F_OFD_SETLK: c_int = 37;
/// Set an OFD-owned file lock; wait if blocked.
pub const F_OFD_SETLKW: c_int = 38;
/// Duplicate file descriptor with the close-on-exec flag `FD_CLOEXEC` set.
pub const F_DUPFD_CLOEXEC: c_int = 1030;

// Used for `l_type` to describe the type of lock {
/// Shared or read lock.
pub const F_RDLCK: c_int = 0;
/// Exclusive or write lock.
pub const F_WRLCK: c_int = 1;
/// Unlock.
pub const F_UNLCK: c_int = 2;
// }

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/creat.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn creat(path: *const c_char, mode: mode_t) -> c_int {
    unsafe { open(path, O_WRONLY | O_CREAT | O_TRUNC, mode) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/fcntl.h.html>.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct flock {
    /// Type of lock; `F_RDLCK`, `F_WRLCK`, `F_UNLCK`.
    pub l_type: c_short,
    /// Flag for starting offset.
    pub l_whence: c_short,
    /// Relative offset in bytes.
    pub l_start: off_t,
    /// Size; if `0` then until EOF.
    pub l_len: off_t,
    /// For a process-owned file lock, ignored on input or the process ID of
    /// the owning process on output; for an OFD-owned file lock, zero on input
    /// or `(pid_t) - 1` on output.
    pub l_pid: pid_t,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fcntl.html>.
///
/// Performs the operation specified by `cmd` on open files pointed to by the
/// file descriptor `fildes`.
///
/// The return value depends on the value of `cmd`:
/// - `F_DUPFD`: A new file descriptor.
/// - `F_DUPFD_CLOEXEC`: A new file descriptor.
/// - `F_DUPFD_CLOFORK`: A new file descriptor.
/// - `F_GETFD`: Value of flags. The return value shall not be negative.
/// - `F_SETFD`: Value other than `-1`.
/// - `F_GETFL`: Value of file status flags and access modes. The return value
///   shall not be negative.
/// - `F_SETFL`: Value other than `-1`.
/// - `F_GETLK`: Value other than `-1`.
/// - `F_SETLK`: Value other than `-1`.
/// - `F_SETLKW`: Value other than `-1`.
/// - `F_OFD_GETLK`: Value other than `-1`.
/// - `F_OFD_SETLK`: Value other than `-1`.
/// - `F_OFD_SETLKW`: Value other than `-1`.
/// - `F_GETOWN`: Value of the socket owner process or process group; this
///   shall not be `-1`.
/// - `F_SETOWN`: Value other than `-1`.
/// - `F_GETOWN_EX`: Value other than `-1`.
/// - `F_SETOWN_EX`: Value other than `-1`.
///
/// Otherwise, `-1` shall be returned and errno set to indicate the error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fcntl(fildes: c_int, cmd: c_int, mut __valist: ...) -> c_int {
    // c_ulonglong
    let arg = match cmd {
        F_DUPFD | F_SETFD | F_SETFL | F_GETLK | F_SETLK | F_SETLKW | F_OFD_GETLK | F_OFD_SETLK
        | F_OFD_SETLKW | F_DUPFD_CLOEXEC => unsafe { __valist.next_arg::<c_ulonglong>() },
        _ => 0,
    };

    Sys::fcntl(fildes, cmd, arg).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/open.html>.
///
/// Establishes a connection between a file and a file descriptor.
///
/// Upon success, opens the file and returns a non-negative integer
/// representing the file descriptor. Upon failure, returns `-1` and sets errno
/// to indicate the error. If `-1` is returned, no files shall be created or
/// modified.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn open(path: *const c_char, oflag: c_int, mut __valist: ...) -> c_int {
    let mode = if oflag & O_CREAT == O_CREAT
    /* || oflag & O_TMPFILE == O_TMPFILE */
    {
        unsafe { __valist.next_arg::<mode_t>() }
    } else {
        0
    };

    let path = unsafe { CStr::from_ptr(path) };
    Sys::openat(AT_FDCWD, path, oflag, mode).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/openat.html>.
///
/// Establishes a connection between a file and a file descriptor. Equivalent
/// to `open()` except in the case where `path` specifies a relative path (the
/// file to be opened is determined relative to the directory associated with
/// the file descriptor `fd` instead of the current working directory).
///
/// Upon success, opens the file and returns a non-negative integer
/// representing the file descriptor. Upon failure, returns `-1` and sets errno
/// to indicate the error. If `-1` is returned, no files shall be created or
/// modified.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openat(
    fd: c_int,
    path: *const c_char,
    oflag: c_int,
    mut __valist: ...
) -> c_int {
    let mode = if oflag & O_CREAT == O_CREAT
    /* || oflag & O_TMPFILE == O_TMPFILE */
    {
        unsafe { __valist.next_arg::<mode_t>() }
    } else {
        0
    };

    let path = unsafe { CStr::from_ptr(path) };
    Sys::openat(fd, path, oflag, mode).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_fallocate.html>.
///
/// Ensures that any required storage for regular file data starting at
/// `offset` and continuing for `len` bytes is allocated on the file system
/// storage media.
///
/// Upon success, returns `0`. Upon failure, returns error number.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_fallocate(fd: c_int, offset: off_t, len: off_t) -> c_int {
    // Length can't be zero and offset must be positive.
    let Ok(offset) = offset.try_into() else {
        return EINVAL;
    };
    let Some(len) = len.try_into().ok().and_then(NonZeroU64::new) else {
        return EINVAL;
    };

    Sys::posix_fallocate(fd, offset, len)
        .err()
        .map(|e| e.0)
        .unwrap_or_default()
}
