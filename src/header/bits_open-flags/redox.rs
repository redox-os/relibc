use crate::platform::types::c_int;

/// Open for reading only.
pub const O_RDONLY: c_int = 0x0001_0000;
/// Open for writing only.
pub const O_WRONLY: c_int = 0x0002_0000;
/// Open for reading and writing.
pub const O_RDWR: c_int = 0x0003_0000;
/// Mask for file access modes.
pub const O_ACCMODE: c_int = 0x0003_0000;
/// Non-blocking mode.
pub const O_NONBLOCK: c_int = 0x0004_0000;
/// Set append mode.
pub const O_APPEND: c_int = 0x0008_0000;
/// Non-POSIX, see <https://man.openbsd.org/open.2>.
///
/// Atomically obtain a shared lock.
pub const O_SHLOCK: c_int = 0x0010_0000;
/// Non-POSIX, see <https://man.openbsd.org/open.2>.
///
/// Atomically obtain an exclusive lock.
pub const O_EXLOCK: c_int = 0x0020_0000;
pub const O_ASYNC: c_int = 0x0040_0000;
pub const O_FSYNC: c_int = 0x0080_0000;
/// Write according to synchronized I/O file integrity completion.
pub const O_SYNC: c_int = O_FSYNC;
/// Atomically set the `FD_CLOEXEC` flag on the new file desciptor.
pub const O_CLOEXEC: c_int = 0x0100_0000;
/// Create file if it does not exist.
pub const O_CREAT: c_int = 0x0200_0000;
/// Truncate flag.
pub const O_TRUNC: c_int = 0x0400_0000;
/// Exclusive use flag.
pub const O_EXCL: c_int = 0x0800_0000;
/// Fail if file is a non-directory file.
pub const O_DIRECTORY: c_int = 0x1000_0000;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/open.2.html>.
///
/// Get a file descriptor to indicate a location in the filesystem tree and
/// to perform operations that act purely at the file descriptor level.
pub const O_PATH: c_int = 0x2000_0000;
pub const O_SYMLINK: c_int = 0x4000_0000;
// Negative to allow it to be used as int
/// Do not follow symbolic links.
pub const O_NOFOLLOW: c_int = -0x8000_0000;

/// Do not assign controlling terminal.
pub const O_NOCTTY: c_int = 0x00000200;

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/open.2.html>.
///
/// Alternative name for `O_NONBLOCK`.
pub const O_NDELAY: c_int = O_NONBLOCK;
