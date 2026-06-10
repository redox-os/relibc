use crate::platform::types::c_int;

/// Close the file descriptor upon execution of an `exec` family function and
/// in the new process image created by `posix_spawn()` or `posix_spawnp()`.
pub const FD_CLOEXEC: c_int = 0x8_0000;

// Flags for capability based "at" functions {
/// Use the current working directory to determine the target of relative file
/// paths.
pub const AT_FDCWD: c_int = -100;
/// Do not follow symbolic links.
pub const AT_SYMLINK_NOFOLLOW: c_int = 0x100;
// AT_EACCESS only used for faccessat
/// Check access using effective user and group ID.
pub const AT_EACCESS: c_int = 0x200;
// AT_REMOVEDIR only used for unlinkat
/// Remove directory instead of file.
pub const AT_REMOVEDIR: c_int = 0x200;
/// Follow symbolic link.
pub const AT_SYMLINK_FOLLOW: c_int = 0x400;
// TODO should be ifdef guarded by _GNU_SOURCE
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/link.2.html>.
pub const AT_EMPTY_PATH: c_int = 0x1000;
// }
