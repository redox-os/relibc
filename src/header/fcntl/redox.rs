use crate::platform::types::c_int;

/// Close the file descriptor upon execution of an `exec` family function and
/// in the new process image created by `posix_spawn()` or `posix_spawnp()`.
pub const FD_CLOEXEC: c_int = 0x0100_0000;

// Flags for capability based "at" functions {
/// Use the current working directory to determine the target of relative file
/// paths.
pub const AT_FDCWD: c_int = -100;
// fchmodat, fchownat, fstatat, utimensat
/// Do not follow symbolic links.
pub const AT_SYMLINK_NOFOLLOW: c_int = 0x200;
// unlinkat
/// Remove directory instead of file.
pub const AT_REMOVEDIR: c_int = 0x200;
// Used by linkat()
/// Follow symbolic link.
pub const AT_SYMLINK_FOLLOW: c_int = 0x2000;
// nonstandard extension, but likely to be in a future standard
// TODO should be ifdef guarded by _GNU_SOURCE
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/link.2.html>.
pub const AT_EMPTY_PATH: c_int = 0x4000;
// only used for faccessat()
/// Check access using effective user and group ID.
pub const AT_EACCESS: c_int = 0x400;
// }
